#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    ptzcam_controller_lib::run();
}
