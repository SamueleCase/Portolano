//! `portolano-app`: the executable of Portolano.app.

// Without it, Windows opens a console window next to a release build.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    portolano_app_lib::run();
}
