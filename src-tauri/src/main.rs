#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;

fn main() {
    if env::args().any(|s| s == "--rdev") {
        bongo_cat_lib::core::rdev_proc::run_child();
        return;
    }
    bongo_cat_lib::run()
}
