use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use log::{info, warn, error};
use tauri::{AppHandle, Emitter, Manager, WebviewWindowBuilder, WebviewUrl};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::oneshot;

use crate::models::options::LauncherOptions;
use crate::services::options_repository::OptionsRepository;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GameState {
    Idle,
    Downloading,
    Running,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameLogLine {
    pub line: String,
    pub is_error: bool,
}

// ---------------------------------------------------------------------------
// GameManager — Tauri managed state
// ---------------------------------------------------------------------------

pub struct GameManager {
    state: Arc<Mutex<GameState>>,
    kill_tx: Arc<tokio::sync::Mutex<Option<oneshot::Sender<()>>>>,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(GameState::Idle)),
            kill_tx: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    pub fn get_state(&self) -> GameState {
        self.state.lock().unwrap().clone()
    }

    pub async fn kill(&self) {
        let mut guard = self.kill_tx.lock().await;
        if let Some(tx) = guard.take() {
            let _ = tx.send(());
        }
    }

    // -----------------------------------------------------------------------
    // Microsoft auth state check
    // -----------------------------------------------------------------------

    /// Checks if a Microsoft token is already stored in the launcher options.
    pub fn has_ms_token(launcher_opts: &LauncherOptions) -> bool {
        launcher_opts.microsoft_token.as_ref().map(|t| !t.is_empty()).unwrap_or(false)
    }

    // -----------------------------------------------------------------------
    // Mods JSON
    // -----------------------------------------------------------------------

    /// Try to download the mods JSON from `MODS_JSON_URL`.
    /// Returns the local path if successful, or `None` if the URL is not set
    /// or the download fails (non-fatal — just means we launch without mods).
    async fn fetch_mods_json(launcher_opts: &LauncherOptions, app: &AppHandle) -> Option<PathBuf> {
        let url = match std::env::var("MODS_JSON_URL") {
            Ok(u) if !u.is_empty() => u,
            _ => return None,
        };

        info!("Fetching mods JSON from {}", url);

        let response = match reqwest::get(&url).await {
            Ok(r) => r,
            Err(e) => {
                warn!("Could not reach mods JSON URL: {}", e);
                return None;
            }
        };

        if !response.status().is_success() {
            warn!("Mods JSON URL returned {}, launching without mods", response.status());
            let _ = app.emit("game-log", GameLogLine {
                line: format!("Mods list not available ({}), launching without mods.", response.status()),
                is_error: false,
            });
            return None;
        }

        let bytes = match response.bytes().await {
            Ok(b) => b,
            Err(e) => {
                warn!("Failed to read mods JSON body: {}", e);
                return None;
            }
        };

        let mods_path = launcher_opts
            .game_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from(".permadeath"))
            .join("mods-list.json");

        if let Some(parent) = mods_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        match std::fs::write(&mods_path, &bytes) {
            Ok(_) => {
                info!("Mods JSON saved to {:?}", mods_path);
                Some(mods_path)
            }
            Err(e) => {
                warn!("Could not save mods JSON: {}", e);
                None
            }
        }
    }

    // -----------------------------------------------------------------------
    // Build CLI argument list
    // -----------------------------------------------------------------------

    fn build_args(
        launcher_opts: &LauncherOptions,
        username: &str,
        jar_path: &Path,
        use_microsoft: bool,
        mods_path: Option<&Path>,
    ) -> Vec<String> {
        let game_opts = OptionsRepository::load_game_options(launcher_opts);

        let game_dir = launcher_opts
            .game_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from(".permadeath"))
            .to_string_lossy()
            .to_string();

        let max_ram = game_opts.get_max_ram();
        // get_vm_flags() appends the GC flag at the end; UnlockExperimentalVMOptions
        // is now the first element of BASE_VM_FLAGS so the order is always correct.
        let jvm_flags_str = game_opts.get_vm_flags().join(" ");

        let mut args: Vec<String> = vec![
            "-jar".to_string(),
            jar_path.to_string_lossy().to_string(),
            "--launcher-dir".to_string(),
            game_dir,
            "--max-ram".to_string(),
            max_ram.to_string(),
        ];

        if !jvm_flags_str.is_empty() {
            args.push("--jvm-args".to_string());
            args.push(jvm_flags_str);
        }

        if let Some(mods) = mods_path {
            args.push("--mods".to_string());
            args.push(mods.to_string_lossy().to_string());
        }

        // Authentication logic:
        // 1. If NOT use_microsoft -> use --offline <username>
        // 2. If use_microsoft AND we have a token -> use --microsoft <token>
        // 3. If use_microsoft AND NO token -> no flag (CLI will open browser)
        if !use_microsoft {
            info!("--offline mode enabled, using username '{}'", username);
            args.push("--offline".to_string());
            args.push(username.to_string());
        } else if let Some(token) = &launcher_opts.microsoft_token {
            info!("--microsoft mode enabled with stored token");
            args.push("--microsoft".to_string());
            args.push(token.clone());
        } else {
            info!("--microsoft mode enabled (no stored token, CLI will open browser)");
        }

        info!("Built CLI arguments: {:?}", args);

        args
    }

    // -----------------------------------------------------------------------
    // Launch
    // -----------------------------------------------------------------------

    pub async fn launch(
        &self,
        username: String,
        use_microsoft: bool,
        app: AppHandle,
    ) -> Result<(), String> {
        if self.get_state() != GameState::Idle {
            return Err("The game is already running or downloading.".to_string());
        }

        let launcher_opts = OptionsRepository::load_launcher_options();
        let jar_path = Self::get_jar_path(&launcher_opts);

        // Download CLI JAR if missing
        if !jar_path.exists() {
            *self.state.lock().unwrap() = GameState::Downloading;
            let _ = app.emit("game-state", GameState::Downloading);
            let _ = app.emit("game-log", GameLogLine {
                line: "Game manager not found. Downloading...".to_string(),
                is_error: false,
            });
            if let Err(e) = Self::download_jar(&jar_path).await {
                *self.state.lock().unwrap() = GameState::Idle;
                let _ = app.emit("game-state", GameState::Idle);
                return Err(e);
            }
            let _ = app.emit("game-log", GameLogLine {
                line: "Download complete. Preparing launch...".to_string(),
                is_error: false,
            });
        }

        // Fetch mods list (non-fatal if unavailable)
        let mods_path = Self::fetch_mods_json(&launcher_opts, &app).await;

        let args = Self::build_args(
            &launcher_opts,
            &username,
            &jar_path,
            use_microsoft,
            mods_path.as_deref(),
        );

        info!("Launching game: java {:?}", args);

        let mut child = tokio::process::Command::new("java")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start the game: {e}. Is Java 21 installed?"))?;

        *self.state.lock().unwrap() = GameState::Running;
        let _ = app.emit("game-state", GameState::Running);

        // Pipe stdout
        let stdout = child.stdout.take().unwrap();
        let app_out = app.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            let mut capturing_auth = false;
            while let Ok(Some(line)) = lines.next_line().await {
                info!("[game] {}", line);

                // Token capturing logic
                if line.trim() == "---AUTH_DATA---" {
                    capturing_auth = true;
                } else if line.trim() == "---END_AUTH_DATA---" {
                    capturing_auth = false;
                } else if capturing_auth && line.starts_with("REFRESH_TOKEN:") {
                    let token = line.replace("REFRESH_TOKEN:", "").trim().to_string();
                    if !token.is_empty() {
                        info!("Microsoft token captured from game output.");
                        let mut opts = OptionsRepository::load_launcher_options();
                        opts.microsoft_token = Some(token);
                        OptionsRepository::save_launcher_options(&opts);
                    }
                }

                let _ = app_out.emit("game-log", GameLogLine { line, is_error: false });
            }
        });

        // Pipe stderr
        let stderr = child.stderr.take().unwrap();
        let app_err = app.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                error!("[game-err] {}", line);
                let _ = app_err.emit("game-log", GameLogLine { line, is_error: true });
            }
        });

        // Kill channel
        let (kill_tx, kill_rx) = oneshot::channel::<()>();
        *self.kill_tx.lock().await = Some(kill_tx);

        let close_launcher = launcher_opts.close_on_launch;
        let show_debug     = launcher_opts.debug_console;

        // Window behaviour:
        // close + debug  → open mini console window, hide main
        // close + !debug → hide main window, no console
        // !close + *     → keep main window open (console visible if debug is on)
        if close_launcher {
            if show_debug {
                // Open a dedicated console window
                if let Err(e) = WebviewWindowBuilder::new(
                    &app,
                    "console",
                    WebviewUrl::App("index.html#/console".into()),
                )
                .title("PERMADEATHSMP — Console")
                .inner_size(1000.0, 600.0)
                .min_inner_size(600.0, 400.0)
                .resizable(true)
                .decorations(true)
                .center()
                .build()
                {
                    error!("Failed to open console window: {}", e);
                } else {
                    // Give it a tiny bit of time to initialize and then send a confirmation log
                    let app_c = app.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        let _ = app_c.emit_to("console", "game-log", GameLogLine {
                            line: "=== Console window initialized. Waiting for logs... ===".into(),
                            is_error: false,
                        });
                    });
                }
            }
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.set_fullscreen(false);
                let _ = win.minimize();
                let _ = win.hide();
            }
        }

        // Wait task
        let state_arc   = Arc::clone(&self.state);
        let kill_tx_arc = Arc::clone(&self.kill_tx);

        tokio::spawn(async move {
            tokio::select! {
                result = child.wait() => {
                    let code = match result {
                        Ok(s)  => s.code().unwrap_or(-1),
                        Err(e) => { error!("Error waiting for game: {}", e); -1 }
                    };
                    info!("Game exited with code {}", code);
                    *state_arc.lock().unwrap() = GameState::Idle;
                    *kill_tx_arc.lock().await = None;
                    let _ = app.emit("game-state", GameState::Idle);
                    let _ = app.emit("game-exited", code);
                    if close_launcher {
                        if let Some(w) = app.get_webview_window("console") { let _ = w.close(); }
                        if let Some(w) = app.get_webview_window("main")    {
                            let _ = w.set_fullscreen(true);
                            let _ = w.unminimize();
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                },
                _ = kill_rx => {
                    info!("Kill signal received — terminating game");
                    let _ = child.kill().await;
                    let _ = child.wait().await;
                    *state_arc.lock().unwrap() = GameState::Idle;
                    *kill_tx_arc.lock().await = None;
                    let _ = app.emit("game-state", GameState::Idle);
                    let _ = app.emit("game-exited", -1i32);
                    if close_launcher {
                        if let Some(w) = app.get_webview_window("console") { let _ = w.close(); }
                        if let Some(w) = app.get_webview_window("main")    {
                            let _ = w.set_fullscreen(true);
                            let _ = w.unminimize();
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                }
            }
        });

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn get_jar_path(launcher_opts: &LauncherOptions) -> PathBuf {
        launcher_opts
            .launcher_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from(".permadeath-launcher"))
            .join("tools")
            .join("minecraft-launcher-cli.jar")
    }

    async fn download_jar(jar_path: &Path) -> Result<(), String> {
        let url = std::env::var("CLI_JAR_URL").map_err(|_| {
            "CLI_JAR_URL is not set in .env — set it to the download URL of the game manager JAR.".to_string()
        })?;

        info!("Downloading CLI JAR from {}", url);

        if let Some(parent) = jar_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create tools directory: {e}"))?;
        }

        let bytes = reqwest::get(&url)
            .await
            .map_err(|e| format!("Download request failed: {e}"))?
            .error_for_status()
            .map_err(|e| format!("Download server returned error: {e}"))?
            .bytes()
            .await
            .map_err(|e| format!("Failed to read download body: {e}"))?;

        std::fs::write(jar_path, &bytes)
            .map_err(|e| format!("Failed to save JAR to disk: {e}"))?;

        info!("CLI JAR saved to {:?}", jar_path);
        Ok(())
    }
}
