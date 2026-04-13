mod algorithm_templates;
mod canvas;
mod config_parser;
mod editor;
mod runner;
mod types;
mod ui;
use std::env;
use std::collections::HashMap;

use crate::types::PackingApp;


fn main() -> iced::Result {
    let args: Vec<String> = env::args().collect();
    let mut lang_map: HashMap<String, bool> = HashMap::from([(String::from("python"), false), (String::from("cpp"), false)]);

    for i in 1..args.len() {
        let temp_str: String = args[i].to_lowercase();
        if lang_map.contains_key(&temp_str) {
            if !lang_map[&temp_str] {
            lang_map.insert(temp_str, true);
            } else {
                break;
            }
        } else {
            break;
        }
    }

    iced::application("Packing App", PackingApp::update, PackingApp::view)
        .theme(|_| iced::Theme::TokyoNight)
        .subscription(PackingApp::subscription)
        .run_with(|| (PackingApp::default(lang_map), iced::Task::none()))
}
