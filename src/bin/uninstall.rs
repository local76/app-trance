use std::path::PathBuf;
use std::fs;
use winapi::um::winuser::{MessageBoxW, MB_YESNO, IDYES, MB_ICONQUESTION, MB_ICONINFORMATION, MB_OK};
use std::os::windows::ffi::OsStrExt;

fn msg_box_yes_no(title: &str, text: &str) -> bool {
    let title_w: Vec<u16> = std::ffi::OsStr::new(title).encode_wide().chain(Some(0)).collect();
    let text_w: Vec<u16> = std::ffi::OsStr::new(text).encode_wide().chain(Some(0)).collect();
    unsafe {
        let result = MessageBoxW(std::ptr::null_mut(), text_w.as_ptr(), title_w.as_ptr(), MB_ICONQUESTION | MB_YESNO);
        result == IDYES
    }
}

fn msg_box(title: &str, text: &str) {
    let title_w: Vec<u16> = std::ffi::OsStr::new(title).encode_wide().chain(Some(0)).collect();
    let text_w: Vec<u16> = std::ffi::OsStr::new(text).encode_wide().chain(Some(0)).collect();
    unsafe {
        MessageBoxW(std::ptr::null_mut(), text_w.as_ptr(), title_w.as_ptr(), MB_ICONINFORMATION | MB_OK);
    }
}

fn unregister_uninstall() -> std::io::Result<()> {
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let subkey_path = r"Software\Microsoft\Windows\CurrentVersion\Uninstall";
    if let Ok(key) = hkcu.open_subkey_with_flags(subkey_path, winreg::enums::KEY_WRITE) {
        let _ = key.delete_subkey("OmaxiScreensaverManager");
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !msg_box_yes_no("Confirm Uninstall", "Are you sure you want to uninstall Screen Saver Management (SSM)?") {
        return Ok(());
    }

    let appdata = std::env::var("APPDATA")?;
    let install_dir = PathBuf::from(&appdata).join(".omaxi").join("apps").join("ssm");

    // Delete manager exe
    let manager_path = install_dir.join("ssm.exe");
    if manager_path.exists() {
        let _ = fs::remove_file(manager_path);
    }

    // Delete configuration file
    let config_path = install_dir.join("config.yaml");
    if config_path.exists() {
        let _ = fs::remove_file(config_path);
    }

    // Delete Start Menu Folder & Shortcuts
    let start_menu_folder = PathBuf::from(&appdata)
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("Omaxi");
        
    if start_menu_folder.exists() {
        let manager_lnk = start_menu_folder.join("Screen Saver Management (SSM).lnk");
        if manager_lnk.exists() {
            let _ = fs::remove_file(manager_lnk);
        }
        let uninstall_lnk = start_menu_folder.join("Uninstall SSM.lnk");
        if uninstall_lnk.exists() {
            let _ = fs::remove_file(uninstall_lnk);
        }
        let _ = fs::remove_dir(&start_menu_folder); // Delete only if empty
    }

    // Remove from Windows Settings Uninstall registry keys
    let _ = unregister_uninstall();

    msg_box("Uninstall Complete", "Screen Saver Management (SSM) has been uninstalled successfully.");

    // Self-delete trick for Windows:
    // We launch cmd in the background to sleep 1 second, delete uninstall.exe, and rmdir the folder.
    let self_exe = std::env::current_exe()?;
    let self_dir = self_exe.parent().unwrap().to_path_buf();
    
    std::process::Command::new("cmd")
        .arg("/C")
        .arg(format!("timeout /T 1 /NOBREAK > NUL && del \"{}\" && rmdir \"{}\"", self_exe.display(), self_dir.display()))
        .spawn()?;

    Ok(())
}
