use std::process::Command;
use log::{info, error};
use launcher_java_installer::JavaSetup;

/// Check if a specific Java version is installed
pub fn check_java_version(target_version: &str) -> bool {
    let output = Command::new("java")
        .arg("-version")
        .output();

    match output {
        Ok(output) => {
            // java -version sends output to stderr, not stdout
            let version_output = String::from_utf8_lossy(&output.stderr);
            version_output.contains(target_version)
        },
        Err(_) => false,
    }
}

/// Show an informational message to the user (platform-specific)
#[cfg(target_os = "windows")]
pub fn show_info_message(message: &str) {
    if let Err(e) = Command::new("cmd")
        .args(&["/C", "start", "cmd", "/C", &format!("echo {} & pause", message)])
        .spawn() {
        error!("No se pudo mostrar ventana informativa: {}", e);
    }
}

#[cfg(not(target_os = "windows"))]
pub fn show_info_message(message: &str) {
    info!("{}", message);
}

/// Ensure Java is installed, attempt installation if not present
pub async fn ensure_java_installed(java_version: &str) -> bool {
    let mut java_installed = check_java_version(java_version);
    info!("Is Java {} installed? {}", java_version, java_installed);

    if !java_installed {
        info!("Java {} not found, trying to install it...", java_version);

        show_info_message(
            &format!("Java {} has not been found on your system, trying to install it... We will notify you once the proccess is completed", java_version)
        );

        let main_disk = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());
        let temp_dir = std::env::var("TEMP").unwrap_or_else(|_| format!("{}\\Temp", main_disk));

        let download_path = format!("{}\\java_download.zip", temp_dir);
        let extract_path = format!("{}\\extracted_java", temp_dir);

        let program_files = std::env::var("ProgramFiles").unwrap_or_else(|_| format!("{}\\Java", main_disk));
        let install_path = format!("{}\\Java\\jdk-{}", program_files, java_version);

        let mut setup = JavaSetup::new(java_version, &download_path, &extract_path, &install_path);

        match setup.setup().await {
            Ok(_) => {
                info!("Java {} installation completed successfully", java_version);
                java_installed = check_java_version(java_version);
                info!("Java {} verification after installation: {}", java_version, java_installed);

                show_info_message(
                    &format!("Java {} have been installed correctly. The Launcher will shutdown, please re-open it.", java_version)
                );
            },
            Err(e) => {
                error!("Error during Java setup: {}", e);
                eprintln!("Error durante la configuración de Java: {}", e);

                show_info_message(
                    &format!("An error occurred during the installation of Java {}. The program will try to continue, but it may not work properly. Even though it might be an error, try re-launching the app. If the problem persists, contact support", java_version)
                );
            }
        }
    }

    java_installed
}
