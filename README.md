# PERMADEATH LAUNCHER - Rewrite

**A launcher for Minecraft PERMADEATH servers, built with [Rust](https://www.rust-lang.org/), [Tauri 2.0](https://tauri.app/), and [Angular](https://angular.dev/).**

This project is a full rewrite of the original [PERMADEATH-LAUNCHER](https://github.com/PERMADEATH-PROJECT/PERMADEATH-LAUNCHER), replacing the vanilla HTML/CSS/TypeScript frontend with an Angular-based architecture. The backend remains Rust + Tauri.

---

## Features

- **Component-based frontend:** UI rebuilt with Angular, providing a structured and maintainable component architecture.
- **Full integration with the Minecraft ecosystem:** Includes Microsoft authentication, mod support, backups, and version control.
- **Modular architecture:** Each part fulfills a separate function and is easily integrable into other Rust projects.

---

## Project Status

### Front-end (Fully completed)
- [x] "Play" Panel
- [x] "Config" Panel
- [x] "Java VM" Panel
- [x] "Updates" Panel
- [x] Log-in Panel

### Back-end (To do)

#### To be implemented in the launcher
- [ ] Server connection (Status, online players, stats, current day, etc)
- [ ] Update control
- [x] Apply config changes to the application
- [x] Apply Java VM changes to the application
- [x] Implement log-in with launcher account
- [x] Implement log-out with launcher account

#### Libraries and Tools

- **[Minecraft CLI Launcher](https://github.com/ponchisao326/Minecraft-Launcher-CLI) (Fully done):**
    - [x] Microsoft auth (OAuth2)
    - [x] Offline mode
    - [x] Minecraft version selection
    - [x] Mod support
    - [x] Automatic updates for Minecraft, Fabric Loader, and mods via FlowUpdater
    - [x] Customizable via command line: game version, RAM, working directory, etc.
    - [x] Progress reporting and detailed logging

- **[Request handler lib](https://github.com/ponchisao326/launcher-request-handler) (Fully done):**
    - [x] Version check: Compares current version to one published at a remote endpoint.
    - [x] Update panel: Retrieves details to display update info (description, changelog, image, etc.) from a remote JSON file.
    - [x] File download: Downloads ZIP files from a URL and stores them at a specified location.
    - [x] JSON utilities: Downloads and deserializes JSON files into any type implementing Deserialize.

- **[Launcher Installer Handler](https://github.com/ponchisao326/launcher-installer-handler) (Fully done):**
    - [x] Flexible extraction: Extracts ZIP files to any directory, ensuring all folders are created as needed.
    - [x] Automatic cleanup: Optionally deletes the ZIP file after extraction.
    - [x] System restart: Optionally restarts the system after installing or updating.
    - [x] Easy integration: Designed for use within Rust-based Minecraft launchers or any application requiring robust ZIP extraction and post-update actions.

- **[Minecraft Launcher Backup Library](https://github.com/ponchisao326/launcher-minecraft-handler) (Fully done):**
    - [x] Customizable backup options: Select which Minecraft folders to back up, set output paths, toggle compression, and exclude certain file extensions.
    - [x] Automatic metadata generation: Each backup generates a JSON file containing timestamp, size, file count, and backup options for tracking and auditing.
    - [x] ZIP compression: Optionally compress backups into a single ZIP file, including the backup metadata.
    - [x] Easy integration: Designed for Rust-based Minecraft launchers or any application needing robust backup functionality.

- **[Java Installation Handler]() (Fully done):**
    - [x] Detect existing Java installations, automatically identifying version numbers, bin paths, and status flags.
    - [x] (Possibly) Apply Java updates automatically, automatically replacing each installed version with a newer release.
    - [x] Prompt users before using a local version, rather than running it by default, unless otherwise specified.
    - [x] Apply the selected version's Java command line options, making those profiles available in a drop down list.
    - [x] Download and install Java versions from the official Oracle website based on the requested version.

---

## Project Structure

```
├── src-tauri/            # Rust backend (Tauri commands, integration logic)
├── src/                  # Angular frontend (components, services, modules)
├── /libs                 # Rust libraries for CLI, backup, installer, etc.
├── /handlers             # Modules and utilities for requests and updates
├── /docs                 # Technical and user documentation
```

---

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [Node.js](https://nodejs.org/)
- [Angular CLI](https://angular.dev/tools/cli)
- [Tauri CLI](https://tauri.app/)

---

## Installation and Usage

1. Install dependencies:
   ```bash
   pnpm install
   cargo install tauri-cli
   ```

2. Run the launcher in development mode:
   ```bash
   pnpm tauri dev
   ```

3. Build for distribution:
   ```bash
   pnpm tauri build
   ```

---

## Community and Contribution

All suggestions, reports, and PRs are welcome.
Check the [issues](https://github.com/PERMADEATH-PROJECT/PERMADEATH-LAUNCHER-REWRITE/issues) for pending tasks and bugs.

---

## License

MIT © ponchisao326
