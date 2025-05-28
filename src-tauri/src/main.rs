#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;

fn main() {
    if env::args().any(|s| s == "--child") {
        bongo_cat_lib::core::child_proc::run_child();
        return;
    }
    bongo_cat_lib::run()
}
