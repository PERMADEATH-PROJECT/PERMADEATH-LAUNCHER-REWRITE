use chrono::Local;
use log::{info, error, LevelFilter};
use simplelog::{WriteLogger, Config, CombinedLogger, TermLogger, TerminalMode, ColorChoice};
use std::fs::{File, create_dir_all};

use crate::models::options::LauncherOptions;

/// Configure the logger to log to both console and a file in the logs directory.
pub fn setup_logger(options: &LauncherOptions) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(dir) = &options.launcher_dir {
        let logs_dir = dir.join("logs");
        create_dir_all(&logs_dir)?;

        let date_str = Local::now().format("%Y-%m-%d").to_string();
        let mut log_filename = format!("{}.log", date_str);
        let mut counter = 1;

        while logs_dir.join(&log_filename).exists() {
            log_filename = format!("{}_{}.log", date_str, counter);
            counter += 1;
        }

        let log_path = logs_dir.join(&log_filename);

        CombinedLogger::init(vec![
            TermLogger::new(
                LevelFilter::Info,
                Config::default(),
                TerminalMode::Mixed,
                ColorChoice::Auto
            ),
            WriteLogger::new(
                LevelFilter::Info,
                Config::default(),
                File::create(log_path)?
            )
        ])?;

        info!("Logger inicializado correctamente en: {}", log_filename);
        Ok(())
    } else {
        error!("No se pudo configurar el directorio de logs");
        Err("Directorio de launcher no configurado".into())
    }
}
