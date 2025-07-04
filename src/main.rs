#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod i18n;
mod ui;
use app::run;
mod core;
mod config;

fn main() {
    run();
}