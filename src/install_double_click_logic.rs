fn install_file_associations() {
    #[cfg(target_os = "windows")]
    {
        // Windows-specific code to create registry entries
        // Usually done via installer, but can be done programmatically
        install_windows_associations();
    }

    #[cfg(target_os = "macos")]
    {
        // macOS-specific code
        // Usually through Info.plist in the app bundle
        install_macos_associations();
    }

    #[cfg(target_os = "linux")]
    {
        // Linux-specific code
        install_linux_associations();
    }
}

#[cfg(target_os = "windows")]
fn install_windows_associations() {
    // Using winreg crate to add registry entries
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);

    // Register .proc.json
    let proc_key = hkcr.create_subkey(".proc.json").unwrap().0;
    proc_key.set_value("", &"ProcedureCrafter.ProcFile").unwrap();

    // Create filetype description
    let proc_file_key =
        hkcr.create_subkey("ProcedureCrafter.ProcFile").unwrap().0;
    proc_file_key.set_value("", &"Procedure Crafter File").unwrap();

    // Set icon
    let icon_key = proc_file_key.create_subkey("DefaultIcon").unwrap().0;
    let exe_path = std::env::current_exe().unwrap();
    icon_key.set_value("", &format!("{},0", exe_path.display())).unwrap();

    // Set open command
    let command_key =
        proc_file_key.create_subkey("shell\\open\\command").unwrap().0;
    command_key
        .set_value("", &format!("\"{}\" \"%1\"", exe_path.display()))
        .unwrap();

    // Similarly for .prop.json
    // ...
}

#[cfg(target_os = "linux")]
fn install_linux_associations() {
    use std::fs;
    use std::io::Write;
    use std::process::Command;

    // Create desktop file
    let home = std::env::var("HOME").unwrap();
    let desktop_file_path =
        format!("{}/.local/share/applications/procedure-crafter.desktop", home);

    let exe_path = std::env::current_exe().unwrap();
    let desktop_content = format!(
        "[Desktop Entry]\n\
        Type=Application\n\
        Name=Procedure Crafter\n\
        Exec=\"{}\" %f\n\
        Icon=procedure-crafter\n\
        Terminal=false\n\
        Categories=Development;\n\
        MimeType=application/x-procedurecrafter;application/x-procedurecrafterplugin;\n",
        exe_path.display()
    );

    let mut file = fs::File::create(&desktop_file_path).unwrap();
    file.write_all(desktop_content.as_bytes()).unwrap();

    // Create MIME type file
    let mime_dir = format!("{}/.local/share/mime/packages", home);
    fs::create_dir_all(&mime_dir).unwrap();
    let mime_file_path =
        format!("{}/application-x-procedurecrafter.xml", mime_dir);

    let mime_content = r#"<?xml version="1.0" encoding="UTF-8"?>
    <mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
      <mime-type type="application/x-procedurecrafter">
        <comment>Procedure Crafter File</comment>
        <glob pattern="*.proc.json"/>
      </mime-type>
      <mime-type type="application/x-procedurecrafterplugin">
        <comment>Procedure Crafter Plugin File</comment>
        <glob pattern="*.prop.json"/>
      </mime-type>
    </mime-info>"#;

    let mut file = fs::File::create(&mime_file_path).unwrap();
    file.write_all(mime_content.as_bytes()).unwrap();

    // Update MIME database
    Command::new("update-mime-database")
        .arg(format!("{}/.local/share/mime", home))
        .output()
        .unwrap();
}

#[cfg(target_os = "macos")]
fn install_macos_associations() {
    // macOS file associations are typically set in the Info.plist
    // of the application bundle, not usually set programmatically
    framework.log("On macOS, file associations should be set in Info.plist");
}
