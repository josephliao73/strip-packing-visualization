mod algorithm_templates;
mod canvas;
mod config_parser;
mod editor;
mod runner;
mod types;
mod ui;

use crate::types::PackingApp;

static INTER: &[u8] = include_bytes!("./Inter.ttc");

fn main() -> iced::Result {
    iced::application("Packing App", PackingApp::update, PackingApp::view)
        .theme(|_| iced::Theme::TokyoNight)
        .subscription(PackingApp::subscription)
        .font(INTER)
        .run_with(|| (PackingApp::default(), iced::Task::none()))
}
