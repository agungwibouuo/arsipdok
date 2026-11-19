// ArsipDok entry point — sebelum Tauri init di lib.rs

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    arsipdok_lib::run()
}
