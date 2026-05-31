use std::path::{Path, PathBuf};
use std::fs;
use winapi::um::winuser::{MessageBoxW, MB_OK, MB_ICONINFORMATION, MB_ICONERROR};
use std::os::windows::ffi::OsStrExt;

fn msg_box(title: &str, text: &str, is_error: bool) {
    let title_w: Vec<u16> = std::ffi::OsStr::new(title).encode_wide().chain(Some(0)).collect();
    let text_w: Vec<u16> = std::ffi::OsStr::new(text).encode_wide().chain(Some(0)).collect();
    let flags = if is_error { MB_ICONERROR } else { MB_ICONINFORMATION } | MB_OK;
    unsafe {
        MessageBoxW(std::ptr::null_mut(), text_w.as_ptr(), title_w.as_ptr(), flags);
    }
}

fn create_shortcut(target: &Path, shortcut_path: &Path, description: &str) -> std::io::Result<()> {
    let script = format!(
        "$WshShell = New-Object -ComObject WScript.Shell; \
         $Shortcut = $WshShell.CreateShortcut('{}'); \
         $Shortcut.TargetPath = '{}'; \
         $Shortcut.Description = '{}'; \
         $Shortcut.Save()",
        shortcut_path.to_string_lossy().replace("'", "''"),
        target.to_string_lossy().replace("'", "''"),
        description.replace("'", "''")
    );
    let status = std::process::Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(&script)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to create shortcut via PowerShell"))
    }
}

fn register_uninstall(install_dir: &Path, manager_path: &Path, uninstall_path: &Path) -> std::io::Result<()> {
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let subkey_path = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\OmaxiScreensaverManager";
    let (key, _) = hkcu.create_subkey(subkey_path)?;
    
    key.set_value("DisplayName", &"Screen Saver Management (SSM)")?;
    key.set_value("UninstallString", &uninstall_path.to_string_lossy().into_owned())?;
    key.set_value("DisplayIcon", &manager_path.to_string_lossy().into_owned())?;
    key.set_value("Publisher", &"Omaxi")?;
    key.set_value("DisplayVersion", &"0.1.0")?;
    key.set_value("InstallLocation", &install_dir.to_string_lossy().into_owned())?;
    Ok(())
}

fn main() {
    let appdata = match std::env::var("APPDATA") {
        Ok(val) => val,
        Err(_) => {
            msg_box("Installation Error", "Could not find %APPDATA% directory.", true);
            return;
        }
    };

    let install_dir = PathBuf::from(appdata).join(".omaxi").join("apps").join("ssm");
    if let Err(e) = fs::create_dir_all(&install_dir) {
        msg_box("Installation Error", &format!("Failed to create installation directory:\n{}", e), true);
        return;
    }

    let current_dir = match std::env::current_exe() {
        Ok(path) => path.parent().unwrap().to_path_buf(),
        Err(_) => PathBuf::from("."),
    };

    let manager_src = current_dir.join("ssm.exe");
    let uninstaller_src = current_dir.join("uninstall.exe");

    if !manager_src.exists() {
        msg_box("Installation Error", "Could not find ssm.exe in the current folder. Make sure to build first.", true);
        return;
    }

    let manager_dst = install_dir.join("ssm.exe");
    let uninstaller_dst = install_dir.join("uninstall.exe");

    // Copy manager
    if let Err(e) = fs::copy(&manager_src, &manager_dst) {
        msg_box("Installation Error", &format!("Failed to copy ssm.exe:\n{}", e), true);
        return;
    }

    // Copy uninstaller (if exists)
    let copied_uninstaller = if uninstaller_src.exists() {
        if let Err(e) = fs::copy(&uninstaller_src, &uninstaller_dst) {
            eprintln!("Warning: Failed to copy uninstall.exe: {}", e);
            false
        } else {
            true
        }
    } else {
        false
    };

    // Create Start Menu Group Folder: Programs/Omaxi
    let start_menu_folder = PathBuf::from(std::env::var("APPDATA").unwrap())
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("Omaxi");
        
    if let Err(e) = fs::create_dir_all(&start_menu_folder) {
        msg_box("Installation Error", &format!("Failed to create Start Menu folder:\n{}", e), true);
        return;
    }

    let shortcut_path = start_menu_folder.join("Screen Saver Management (SSM).lnk");
    let shortcut_success = create_shortcut(&manager_dst, &shortcut_path, "Manage and configure Omaxi Screensavers").is_ok();

    let mut uninstall_shortcut_success = false;
    if copied_uninstaller {
        let uninstall_shortcut_path = start_menu_folder.join("Uninstall SSM.lnk");
        uninstall_shortcut_success = create_shortcut(&uninstaller_dst, &uninstall_shortcut_path, "Uninstall Screen Saver Management").is_ok();
    }

    // Register with Windows Settings (Add or Remove Programs)
    let reg_success = register_uninstall(&install_dir, &manager_dst, &uninstaller_dst).is_ok();

    let mut msg = format!(
        "Screen Saver Management (SSM) has been successfully installed!\n\n\
         Destination: {}\n\n",
        install_dir.display()
    );

    if shortcut_success {
        msg.push_str("✓ App shortcut added to Start Menu (Programs\\Omaxi).\n");
    }
    if uninstall_shortcut_success {
        msg.push_str("✓ Uninstaller shortcut added to Start Menu.\n");
    }
    if reg_success {
        msg.push_str("✓ Registered with Windows Apps & Features for quick uninstallation.\n");
    }

    msg_box("Installation Complete", &msg, false);
}
