#![cfg_attr(all(not(debug_assertions)), windows_subsystem = "windows")]

fn main() {
    permadeath_launcher_lib::run();
}