use iced::widget::{button, column, container, row, scrollable, text, text_editor};
use iced::highlighter::Theme as HighlighterTheme;
use iced::{Element, Theme, Alignment, Length, Color, Font};
use crate::types::{BottomPanelTab, CodeLanguage, Input, JsonInput};

pub struct EditorState<'a> {
    pub code_editor_content: &'a text_editor::Content,
    pub selected_language: CodeLanguage,
    pub bottom_panel_visible: bool,
    pub bottom_panel_tab: BottomPanelTab,
    pub code_errors: &'a [String],
    pub code_output_json: Option<&'a str>,
    pub show_visualization_button: bool,
    pub testcase_message: Option<&'a str>,
    pub testcase: Option<&'a JsonInput>,
    pub is_root: bool,  // True for root tab, false for nodes (nodes don't have test cases)
}

fn build_language_selector(language: CodeLanguage) -> Element<'static, Input> {
    let label = match language {
        CodeLanguage::Python => "Python",
        CodeLanguage::Cpp => "C++",
        CodeLanguage::Java => "Java",
    };

    let language_label = text(label)
        .size(12)
        .font(Font::default())
        .style(|_theme: &Theme| {
            text::Style {
                color: Some(Color::from_rgb(0.55, 0.55, 0.6)),
            }
        });

    container(language_label)
        .padding([6, 12])
        .style(|_theme: &Theme| {
            container::Style {
                background: Some(Color::from_rgb(0.1, 0.1, 0.13).into()),
                border: iced::Border {
                    color: Color::from_rgb(0.2, 0.2, 0.26),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

fn build_save_output_json_button(enabled: bool) -> Element<'static, Input> {
    let ui_font = Font::default();

    button(
        row![
            text("Save Output Json").size(11).font(ui_font),
        ]
        .spacing(4)
        .align_y(Alignment::Center)
    )
    .padding([6, 12])
    .style(move |_theme: &Theme, status| {
        let bg = if enabled {
            match status {
                button::Status::Hovered => Color::from_rgb(0.18, 0.18, 0.22),
                _ => Color::from_rgb(0.14, 0.14, 0.17),
            }
        } else {
            Color::from_rgb(0.10, 0.10, 0.12)
        };

        let border = if enabled {
            Color::from_rgb(0.24, 0.24, 0.28)
        } else {
            Color::from_rgb(0.16, 0.16, 0.19)
        };

        let text_color = if enabled {
            Color::from_rgb(0.82, 0.82, 0.85)
        } else {
            Color::from_rgb(0.45, 0.45, 0.50)
        };

        button::Style {
            background: Some(bg.into()),
            border: iced::Border {
                color: border,
                width: 1.0,
                radius: 4.0.into(),
            },
            text_color,
            ..Default::default()
        }
    })
    .into()
}

fn build_show_visualization_button(enabled: bool) -> Element<'static, Input> {
    let ui_font = Font::default();

    button(
        row![
            text("Show Visualization").size(11).font(ui_font),
        ]
        .spacing(4)
        .align_y(Alignment::Center)
    )
    .padding([6, 12])
    .style(move |_theme: &Theme, status| {
        let bg = if enabled {
            match status {
                button::Status::Hovered => Color::from_rgb(0.18, 0.18, 0.22),
                _ => Color::from_rgb(0.14, 0.14, 0.17),
            }
        } else {
            Color::from_rgb(0.10, 0.10, 0.12)
        };

        let border = if enabled {
            Color::from_rgb(0.24, 0.24, 0.28)
        } else {
            Color::from_rgb(0.16, 0.16, 0.19)
        };

        let text_color = if enabled {
            Color::from_rgb(0.82, 0.82, 0.85)
        } else {
            Color::from_rgb(0.45, 0.45, 0.50)
        };

        button::Style {
            background: Some(bg.into()),
            border: iced::Border {
                color: border,
                width: 1.0,
                radius: 4.0.into(),
            },
            text_color,
            ..Default::default()
        }
    })
    .into()
}


fn build_run_button() -> Element<'static, Input> {
    button(
        container(
            text("Run")
                .size(12)
                .font(Font::default())
        )
        .center_x(Length::Fill)
    )
    .on_press(Input::RunCode)
    .padding([8, 24])
    .style(|_theme: &Theme, status| {
        let base_bg = Color::from_rgb(0.2, 0.5, 0.3);
        let hover_bg = Color::from_rgb(0.25, 0.6, 0.35);

        button::Style {
            background: Some(match status {
                button::Status::Hovered => hover_bg.into(),
                _ => base_bg.into(),
            }),
            border: iced::Border {
                color: Color::from_rgb(0.3, 0.6, 0.4),
                width: 1.0,
                radius: 6.0.into(),
            },
            text_color: Color::from_rgb(0.95, 0.95, 0.97),
            ..Default::default()
        }
    })
    .into()
}

fn build_code_editor<'a>(
    content: &'a text_editor::Content,
    language: CodeLanguage,
) -> Element<'a, Input> {
    let highlight_ext = match language {
        CodeLanguage::Python => "py",
        CodeLanguage::Cpp => "cpp",
        CodeLanguage::Java => "java",
    };

    let code_editor = text_editor(content)
        .on_action(Input::CodeEditorAction)
        .height(Length::Fill)
        .padding(12)
        .size(13)
        .font(Font::MONOSPACE)
        .highlight(highlight_ext, HighlighterTheme::Base16Eighties);

    container(code_editor)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme: &Theme| {
            container::Style {
                background: Some(Color::from_rgb(0.05, 0.05, 0.07).into()),
                border: iced::Border {
                    color: Color::from_rgb(0.16, 0.16, 0.2),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}


fn build_output_tab_button(is_active: bool) -> Element<'static, Input> {
    let ui_font = Font::default();

    button(
        text("Output").size(11).font(ui_font)
    )
    .on_press(Input::BottomPanelTabSelected(BottomPanelTab::Output))
    .padding([6, 12])
    .style(move |_theme: &Theme, status| {
        let bg = match status {
            button::Status::Hovered => Color::from_rgb(0.12, 0.12, 0.15),
            _ => Color::TRANSPARENT,
        };
        button::Style {
            background: Some(bg.into()),
            border: iced::Border {
                color: if is_active {
                    Color::from_rgb(0.4, 0.4, 0.45)
                } else {
                    Color::TRANSPARENT
                },
                width: if is_active { 1.0 } else { 0.0 },
                radius: 4.0.into(),
            },
            text_color: if is_active {
                Color::from_rgb(0.9, 0.9, 0.92)
            } else {
                Color::from_rgb(0.5, 0.5, 0.55)
            },
            ..Default::default()
        }
    })
    .into()
}

fn build_test_cases_tab_button(is_active: bool) -> Element<'static, Input> {
    let ui_font = Font::default();

    button(
        text("Test Cases").size(11).font(ui_font)
    )
    .on_press(Input::BottomPanelTabSelected(BottomPanelTab::TestCases))
    .padding([6, 12])
    .style(move |_theme: &Theme, status| {
        let bg = match status {
            button::Status::Hovered => Color::from_rgb(0.12, 0.12, 0.15),
            _ => Color::TRANSPARENT,
        };
        button::Style {
            background: Some(bg.into()),
            border: iced::Border {
                color: if is_active {
                    Color::from_rgb(0.4, 0.4, 0.45)
                } else {
                    Color::TRANSPARENT
                },
                width: if is_active { 1.0 } else { 0.0 },
                radius: 4.0.into(),
            },
            text_color: if is_active {
                Color::from_rgb(0.9, 0.9, 0.92)
            } else {
                Color::from_rgb(0.5, 0.5, 0.55)
            },
            ..Default::default()
        }
    })
    .into()
}

fn build_toggle_panel_button(is_visible: bool) -> Element<'static, Input> {
    let ui_font = Font::default();

    button(
        text(if is_visible { "▼" } else { "▲" })
            .size(10)
            .font(ui_font)
    )
    .on_press(Input::ToggleBottomPanel)
    .padding([4, 8])
    .style(|_theme: &Theme, status| {
        let bg = match status {
            button::Status::Hovered => Color::from_rgb(0.15, 0.15, 0.18),
            _ => Color::TRANSPARENT,
        };
        button::Style {
            background: Some(bg.into()),
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            text_color: Color::from_rgb(0.6, 0.6, 0.65),
            ..Default::default()
        }
    })
    .into()
}

fn build_bottom_panel_tab_bar<'a>(
    active_tab: BottomPanelTab,
    is_visible: bool,
    show_viz_button: bool,
    is_root: bool,
) -> Element<'a, Input> {
    let output_tab_active = active_tab == BottomPanelTab::Output;
    let test_cases_tab_active = active_tab == BottomPanelTab::TestCases;

    let mut tab_row = row![build_output_tab_button(output_tab_active)];

    // Only show Test Cases tab for root (nodes use inherited region rectangles)
    if is_root {
        tab_row = tab_row.push(build_test_cases_tab_button(test_cases_tab_active));
    }

    tab_row = tab_row
        .push(column![].width(Length::Fill))
        .push(build_toggle_panel_button(is_visible))
        .push(build_show_visualization_button(show_viz_button))
        .push(build_save_output_json_button(show_viz_button));

    container(tab_row.spacing(4).align_y(Alignment::Center))
    .padding([4, 8])
    .width(Length::Fill)
    .style(|_theme: &Theme| {
        container::Style {
            background: Some(Color::from_rgb(0.055, 0.055, 0.07).into()),
            border: iced::Border {
                color: Color::from_rgb(0.12, 0.12, 0.15),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }
    })
    .into()
}

fn build_save_output_button() -> Element<'static, Input> {
    let ui_font = Font::default();

    button(
        row![
            text("Save to File").size(11).font(ui_font),
        ].spacing(4).align_y(Alignment::Center)
    )
    .on_press(Input::SaveOutputToFile)
    .padding([6, 12])
    .style(|_theme: &Theme, status| {
        let bg = match status {
            button::Status::Hovered => Color::from_rgb(0.18, 0.18, 0.22),
            _ => Color::from_rgb(0.14, 0.14, 0.17),
        };
        button::Style {
            background: Some(bg.into()),
            border: iced::Border {
                color: Color::from_rgb(0.24, 0.24, 0.28),
                width: 1.0,
                radius: 4.0.into(),
            },
            text_color: Color::from_rgb(0.8, 0.8, 0.83),
            ..Default::default()
        }
    })
    .into()
}

fn build_output_content<'a>(json: Option<&'a str>) -> Element<'a, Input> {
    let ui_font = Font::default();

    if let Some(json) = json {
        column![
            container(
                row![
                    text("JSON Output").size(11).font(ui_font).style(|_theme: &Theme| {
                        text::Style {
                            color: Some(Color::from_rgb(0.6, 0.6, 0.65)),
                        }
                    }),
                    column![].width(Length::Fill),
                    build_save_output_button(),
                ]
                .spacing(8)
                .align_y(Alignment::Center)
            )
            .padding([8, 10])
            .width(Length::Fill)
            .style(|_theme: &Theme| {
                container::Style {
                    background: Some(Color::from_rgb(0.055, 0.055, 0.07).into()),
                    border: iced::Border {
                        color: Color::from_rgb(0.12, 0.12, 0.15),
                        width: 0.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                }
            }),
            scrollable(
                container(
                    text(json.to_string())
                        .size(11)
                        .font(Font::MONOSPACE)
                        .style(|_theme: &Theme| {
                            text::Style {
                                color: Some(Color::from_rgb(0.75, 0.85, 0.75)),
                            }
                        })
                )
                .padding(10)
                .width(Length::Fill)
            )
            .height(Length::Fill),
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        container(
            text("No output yet. Run your code to see results.")
                .size(12)
                .font(ui_font)
                .style(|_theme: &Theme| {
                    text::Style {
                        color: Some(Color::from_rgb(0.45, 0.45, 0.5)),
                    }
                })
        )
        .padding(12)
        .width(Length::Fill)
        .into()
    }
}

fn build_test_cases_content<'a>(message: Option<&'a str>, testcase: Option<&'a JsonInput>) -> Element<'a, Input> {
    let ui_font = Font::default();

    let import_button = button(
        text("Import Test Case").size(11).font(ui_font)
    )
    .on_press(Input::ImportTestCase)
    .padding([8, 16])
    .style(|_theme: &Theme, status| {
        let bg = match status {
            button::Status::Hovered => Color::from_rgb(0.18, 0.18, 0.22),
            _ => Color::from_rgb(0.14, 0.14, 0.17),
        };
        button::Style {
            background: Some(bg.into()),
            border: iced::Border {
                color: Color::from_rgb(0.24, 0.24, 0.28),
                width: 1.0,
                radius: 4.0.into(),
            },
            text_color: Color::from_rgb(0.8, 0.8, 0.83),
            ..Default::default()
        }
    });

    let generate_button = button(
        text("Generate Random").size(11).font(ui_font)
    )
    .on_press(Input::GenerateTestCase)
    .padding([8, 16])
    .style(|_theme: &Theme, status| {
        let bg = match status {
            button::Status::Hovered => Color::from_rgb(0.2, 0.35, 0.5),
            _ => Color::from_rgb(0.15, 0.28, 0.42),
        };
        button::Style {
            background: Some(bg.into()),
            border: iced::Border {
                color: Color::from_rgb(0.25, 0.4, 0.55),
                width: 1.0,
                radius: 4.0.into(),
            },
            text_color: Color::from_rgb(0.85, 0.9, 0.95),
            ..Default::default()
        }
    });

    let message_element: Element<'a, Input> = if let Some(msg) = message {
        let is_success = msg.starts_with("✓");
        let text_color = if is_success {
            Color::from_rgb(0.4, 0.85, 0.5)
        } else {
            Color::from_rgb(1.0, 0.55, 0.55)
        };
        text(msg)
            .size(11)
            .font(ui_font)
            .style(move |_theme: &Theme| text::Style { color: Some(text_color) })
            .into()
    } else {
        text("No test case loaded")
            .size(11)
            .font(ui_font)
            .style(|_theme: &Theme| text::Style { color: Some(Color::from_rgb(0.5, 0.5, 0.55)) })
            .into()
    };

    let testcase_display: Element<'a, Input> = if let Some(tc) = testcase {
        let mut display_text = format!("Bin Width: {}\nRectangles: ", tc.width_of_bin);
        for (i, rect) in tc.rectangle_list.iter().take(10).enumerate() {
            if i > 0 {
                display_text.push_str(", ");
            }
            display_text.push_str(&format!("{}x{}({})", rect.width, rect.height, rect.quantity));
        }
        if tc.rectangle_list.len() > 10 {
            display_text.push_str(&format!(" ... +{} more", tc.rectangle_list.len() - 10));
        }

        scrollable(
            text(display_text)
                .size(10)
                .font(Font::MONOSPACE)
                .style(|_theme: &Theme| text::Style {
                    color: Some(Color::from_rgb(0.7, 0.75, 0.8))
                })
        )
        .height(Length::Fill)
        .into()
    } else {
        column![].into()
    };

    container(
        column![
            row![
                import_button,
                generate_button,
                column![].width(Length::Fill),
                message_element,
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            column![].height(8),
            testcase_display,
        ]
        .spacing(0)
        .height(Length::Fill)
    )
    .padding(12)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn build_bottom_panel<'a>(state: &EditorState<'a>) -> Element<'a, Input> {
    let tab_bar = build_bottom_panel_tab_bar(
        state.bottom_panel_tab,
        state.bottom_panel_visible,
        state.show_visualization_button,
        state.is_root,
    );

    if state.bottom_panel_visible {
        let content: Element<'_, Input> = match state.bottom_panel_tab {
            BottomPanelTab::Output | BottomPanelTab::Problems => {
                build_output_content(state.code_output_json)
            }
            BottomPanelTab::TestCases => {
                build_test_cases_content(state.testcase_message, state.testcase)
            }
        };

        container(
            column![
                tab_bar,
                container(content)
                    .width(Length::Fill)
                    .height(Length::Fixed(150.0))
                    .style(|_theme: &Theme| {
                        container::Style {
                            background: Some(Color::from_rgb(0.045, 0.045, 0.055).into()),
                            border: iced::Border {
                                color: Color::from_rgb(0.12, 0.12, 0.15),
                                width: 1.0,
                                radius: 6.0.into(),
                            },
                            ..Default::default()
                        }
                    }),
            ]
            .spacing(0)
        )
        .width(Length::Fill)
        .into()
    } else {
        container(tab_bar)
            .width(Length::Fill)
            .into()
    }
}

pub fn build_code_panel<'a>(state: &EditorState<'a>) -> Element<'a, Input> {
    let language_selector = build_language_selector(state.selected_language);
    let run_button = build_run_button();
    let code_editor = build_code_editor(state.code_editor_content, state.selected_language);
    let bottom_panel = build_bottom_panel(state);

    column![
        row![
            language_selector,
            column![].width(Length::Fill),
            run_button,
        ].spacing(8).align_y(Alignment::Center),
        column![].height(8),
        code_editor,
        column![].height(8),
        bottom_panel,
    ]
    .spacing(0)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
