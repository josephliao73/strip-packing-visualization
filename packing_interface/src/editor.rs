use iced::widget::{button, column, container, row, scrollable, text, text_editor, text_input};
use serde_json;
use iced::highlighter::Theme as HighlighterTheme;
use iced::{Element, Theme, Alignment, Length, Color, Font};
use crate::types::{BottomPanelTab, CodeLanguage, Input, JsonInput, MultipleRunResult};

pub struct EditorState<'a> {
    pub code_editor_content: &'a text_editor::Content,
    pub selected_language: CodeLanguage,
    pub bottom_panel_visible: bool,
    pub bottom_panel_tab: BottomPanelTab,
    pub code_errors: &'a [String],
    pub code_output_json: Option<&'a str>,
    pub testcase_message: Option<&'a str>,
    pub multiple_testcase_message: Option<&'a str>,
    pub testcase: Option<&'a JsonInput>,
    pub is_root: bool,  // True for root tab, false for nodes (nodes don't have test cases)
    pub has_single_testcase: bool,
    pub has_multiple_testcases: bool,
    pub num_test_cases_input: &'a str,
    pub input_size_input: &'a str,
    pub unique_types_input: &'a str,
    pub multiple_run_results: &'a [MultipleRunResult],
    pub multiple_results_expanded: &'a [bool],
    pub bottom_panel_height: f32,
}

fn build_language_selector(language: CodeLanguage) -> Element<'static, Input> {
    let label = match language {
        CodeLanguage::Python => "Python",
        CodeLanguage::Cpp => "C++",
        CodeLanguage::Java => "Java",
    };

    let language_label = text(label)
        .size(13)
        .font(Font::default())
        .style(|_theme: &Theme| {
            text::Style {
                color: Some(Color::from_rgb(0.62, 0.65, 0.76)),
            }
        });

    container(language_label)
        .padding([8, 14])
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
            text("Save Results JSON").size(11).font(ui_font),
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

fn build_run_button(has_testcase: i32) -> Element<'static, Input> {
    let mut btn = button(
        container(
            text("Run")
                .size(14)
                .font(Font::default())
        )
        .center_x(Length::Fill)
    );

    if has_testcase == 1 || has_testcase == 2 {
        btn = btn.on_press(Input::RunCode(has_testcase));
    }

    btn.padding([10, 30])
    .style(move |_theme: &Theme, status| {
        let (base_bg, hover_bg, text_color) = if has_testcase != 0 {
            (Color::from_rgb(0.18, 0.48, 0.28), Color::from_rgb(0.22, 0.58, 0.34), Color::from_rgb(0.96, 0.98, 0.96))
        } else {
            (Color::from_rgb(0.12, 0.12, 0.15), Color::from_rgb(0.12, 0.12, 0.15), Color::from_rgb(0.38, 0.38, 0.43))
        };
        button::Style {
            background: Some(match status {
                button::Status::Hovered => hover_bg.into(),
                _ => base_bg.into(),
            }),
            border: iced::Border {
                color: if has_testcase != 0 { Color::from_rgb(0.28, 0.58, 0.38) } else { Color::from_rgb(0.18, 0.18, 0.22) },
                width: 1.0,
                radius: 8.0.into(),
            },
            text_color,
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


fn build_results_header(show_batch_results: bool) -> Element<'static, Input> {
    let ui_font = Font::default();

    container(
        row![
            text(if show_batch_results { "Batch Results" } else { "Results" })
                .size(12)
                .font(ui_font)
                .style(|_theme: &Theme| text::Style {
                    color: Some(Color::from_rgb(0.9, 0.9, 0.92)),
                }),
            column![].width(Length::Fill),
            text(if show_batch_results { "Inspect generated batch runs" } else { "Latest run output" })
                .size(10)
                .font(ui_font)
                .style(|_theme: &Theme| text::Style {
                    color: Some(Color::from_rgb(0.5, 0.5, 0.55)),
                }),
        ]
        .align_y(Alignment::Center)
    )
    .padding([8, 10])
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

fn build_multiple_test_cases_content<'a>(
    message: Option<&'a str>,
    num_test_cases: &'a str,
    input_size: &'a str,
    unique_types: &'a str,
) -> Element<'a, Input> {
    let ui_font = Font::default();

    let n_input = text_input("Number of test cases", num_test_cases)
        .on_input(Input::NumTestCasesChanged)
        .size(11)
        .padding(8)
        .width(Length::Fixed(120.0));

    let input_size_placeholder = if input_size.is_empty() { "100" } else { "" };
    let input_size_field = text_input(input_size_placeholder, input_size)
        .on_input(Input::InputSizeChanged)
        .size(11)
        .padding(8)
        .width(Length::Fixed(90.0))
        .style(move |theme: &Theme, status| {
            let mut style = text_input::default(theme, status);
            if input_size.is_empty() {
                style.value = Color::from_rgb(0.38, 0.38, 0.44);
            }
            style
        });

    let input_size_default_label: Element<'a, Input> = if input_size.is_empty() {
        text("(default)")
            .size(10)
            .font(ui_font)
            .style(|_theme: &Theme| text::Style {
                color: Some(Color::from_rgb(0.35, 0.35, 0.42)),
            })
            .into()
    } else {
        column![].into()
    };

    let unique_types_field = text_input("Any", unique_types)
        .on_input(Input::UniqueTypesChanged)
        .size(11)
        .padding(8)
        .width(Length::Fixed(90.0));

    let count = num_test_cases.parse::<i32>().unwrap_or(0);
    let enabled = count > 0;
    let mut generate_button = button(
        text("Generate Test Cases").size(11).font(ui_font)
    )
    .padding([8, 16])
    .style(move |_theme: &Theme, status| {
        let (bg, text_color) = if enabled {
            (match status {
                button::Status::Hovered => Color::from_rgb(0.2, 0.35, 0.5),
                _ => Color::from_rgb(0.15, 0.28, 0.42),
            }, Color::from_rgb(0.85, 0.9, 0.95))
        } else {
            (Color::from_rgb(0.12, 0.12, 0.15), Color::from_rgb(0.4, 0.4, 0.45))
        };
        button::Style {
            background: Some(bg.into()),
            border: iced::Border {
                color: if enabled { Color::from_rgb(0.25, 0.4, 0.55) } else { Color::from_rgb(0.18, 0.18, 0.22) },
                width: 1.0,
                radius: 4.0.into(),
            },
            text_color,
            ..Default::default()
        }
    });
    if enabled {
        generate_button = generate_button.on_press(Input::GenerateMultipleTestCases(count));
    }

    let dim_label = |label: &'static str| {
        text(label)
            .size(11)
            .font(Font::default())
            .style(|_theme: &Theme| text::Style {
                color: Some(Color::from_rgb(0.5, 0.5, 0.55)),
            })
    };

    container(
        column![
            row![
                dim_label("Count:"),
                n_input,
                column![].width(16),
                dim_label("Input size:"),
                input_size_field,
                input_size_default_label,
                column![].width(16),
                dim_label("Unique types:"),
                unique_types_field,
            ]
            .spacing(6)
            .align_y(Alignment::Center),
            row![generate_button].align_y(Alignment::Center),
            text(message.unwrap_or("No test cases generated"))
                .size(11)
                .font(ui_font)
                .style(|_theme: &Theme| text::Style {
                    color: Some(Color::from_rgb(0.5, 0.5, 0.55)),
                }),
        ]
        .spacing(8)
    )
    .padding(12)
    .width(Length::Fill)
    .into()
}

fn build_test_cases_content<'a>(message: Option<&'a str>, testcase: Option<&'a JsonInput>, input_size: &'a str, unique_types: &'a str) -> Element<'a, Input> {
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

    let input_size_placeholder = if input_size.is_empty() { "100" } else { "" };
    let input_size_field = text_input(input_size_placeholder, input_size)
        .on_input(Input::InputSizeChanged)
        .size(11)
        .padding(8)
        .width(Length::Fixed(90.0))
        .style(move |theme: &Theme, status| {
            let mut style = text_input::default(theme, status);
            if input_size.is_empty() {
                style.value = Color::from_rgb(0.38, 0.38, 0.44);
            }
            style
        });

    let input_size_default_label: Element<'a, Input> = if input_size.is_empty() {
        text("(default)")
            .size(10)
            .font(ui_font)
            .style(|_theme: &Theme| text::Style {
                color: Some(Color::from_rgb(0.35, 0.35, 0.42)),
            })
            .into()
    } else {
        column![].into()
    };

    let unique_types_field = text_input("Any", unique_types)
        .on_input(Input::UniqueTypesChanged)
        .size(11)
        .padding(8)
        .width(Length::Fixed(90.0));

    let dim_label = |label: &'static str| {
        text(label)
            .size(11)
            .font(Font::default())
            .style(|_theme: &Theme| text::Style {
                color: Some(Color::from_rgb(0.5, 0.5, 0.55)),
            })
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
            row![
                dim_label("Input size:"),
                input_size_field,
                input_size_default_label,
                column![].width(16),
                dim_label("Unique types:"),
                unique_types_field,
            ]
            .spacing(6)
            .align_y(Alignment::Center),
            column![].height(4),
            testcase_display,
        ]
        .spacing(6)
        .height(Length::Fill)
    )
    .padding(12)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn build_multi_run_results_content<'a>(
    results: &'a [MultipleRunResult],
    expanded: &'a [bool],
) -> Element<'a, Input> {
    let ui_font = Font::default();

    let valid_heights: Vec<f32> = results.iter().filter_map(|r| r.height).collect();
    let avg_str = if valid_heights.is_empty() {
        "N/A".to_string()
    } else {
        format!("{:.2}", valid_heights.iter().sum::<f32>() / valid_heights.len() as f32)
    };

    let header = container(
        row![
            text(format!(
                "Average height: {}   ({}/{} succeeded)",
                avg_str,
                valid_heights.len(),
                results.len()
            ))
            .size(11)
            .font(ui_font)
            .style(|_theme: &Theme| text::Style {
                color: Some(Color::from_rgb(0.75, 0.85, 0.75)),
            }),
        ]
        .align_y(Alignment::Center)
    )
    .padding([8, 10])
    .width(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(Color::from_rgb(0.055, 0.055, 0.07).into()),
        border: iced::Border {
            color: Color::from_rgb(0.12, 0.12, 0.15),
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    });

    let mut rows: Vec<Element<'a, Input>> = Vec::new();
    for (i, result) in results.iter().enumerate() {
        let is_expanded = expanded.get(i).copied().unwrap_or(false);
        let height_str = result.height
            .map(|h| format!("{:.2}", h))
            .unwrap_or_else(|| "Error".to_string());
        let arrow = if is_expanded { "▼" } else { "▶" };
        let label = format!("{}  Test Case {} - Height: {}", arrow, i + 1, height_str);

        let row_btn = button(
            text(label).size(11).font(ui_font)
        )
        .on_press(Input::ToggleMultipleResultExpanded(i))
        .padding([4, 10])
        .width(Length::Fill)
        .style(|_theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered => Color::from_rgb(0.10, 0.10, 0.13),
                _ => Color::TRANSPARENT,
            };
            button::Style {
                background: Some(bg.into()),
                border: iced::Border {
                    color: Color::from_rgb(0.14, 0.14, 0.18),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                text_color: Color::from_rgb(0.75, 0.80, 0.85),
                ..Default::default()
            }
        });

        let has_output = result.output.is_some();
        let display_btn = button(
            text("Display").size(10).font(ui_font)
        )
        .padding([3, 8])
        .style(move |_theme: &Theme, status| {
            let (bg, text_color) = if has_output {
                (match status {
                    button::Status::Hovered => Color::from_rgb(0.18, 0.30, 0.45),
                    _ => Color::from_rgb(0.12, 0.22, 0.35),
                }, Color::from_rgb(0.75, 0.88, 1.0))
            } else {
                (Color::from_rgb(0.10, 0.10, 0.12), Color::from_rgb(0.35, 0.35, 0.40))
            };
            button::Style {
                background: Some(bg.into()),
                border: iced::Border {
                    color: if has_output { Color::from_rgb(0.22, 0.38, 0.55) } else { Color::from_rgb(0.15, 0.15, 0.18) },
                    width: 1.0,
                    radius: 4.0.into(),
                },
                text_color,
                ..Default::default()
            }
        });
        let display_btn = if has_output {
            display_btn.on_press(Input::DisplayMultipleResult(i))
        } else {
            display_btn
        };

        rows.push(row_btn.into());

        if is_expanded {
            let json = serde_json::to_string_pretty(&result.testcase)
                .unwrap_or_else(|_| "{}".to_string());
            rows.push(
                container(
                    column![
                        display_btn,
                        text(json)
                            .size(10)
                            .font(Font::MONOSPACE)
                            .style(|_theme: &Theme| text::Style {
                                color: Some(Color::from_rgb(0.65, 0.75, 0.70)),
                            }),
                    ]
                    .spacing(6)
                )
                .padding([4, 16])
                .width(Length::Fill)
                .into()
            );
        }
    }

    let result_list = column(rows).spacing(2).width(Length::Fill);

    column![
        header,
        scrollable(
            container(result_list)
                .padding([4, 4])
                .width(Length::Fill)
        )
        .height(Length::Fill),
    ]
    .spacing(0)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn build_bottom_panel<'a>(state: &EditorState<'a>) -> Element<'a, Input> {
    let show_batch_results = !state.multiple_run_results.is_empty();
    let content: Element<'_, Input> = if show_batch_results {
        build_multi_run_results_content(state.multiple_run_results, state.multiple_results_expanded)
    } else {
        build_output_content(state.code_output_json)
    };

    container(
        column![
            build_results_header(show_batch_results),
            container(content)
                .width(Length::Fill)
                .height(220)
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
        .spacing(6)
    )
    .width(Length::Fill)
    .into()
}

pub fn build_code_panel<'a>(state: &EditorState<'a>) -> Element<'a, Input> {
    let language_selector = build_language_selector(state.selected_language);
    let has_testcase: i32 = if !state.is_root {
        1
    } else if state.has_single_testcase {
        1
    } else if state.has_multiple_testcases {
        2
    } else {
        0
    };
    let run_button = build_run_button(has_testcase);
    let code_editor = build_code_editor(state.code_editor_content, state.selected_language);
    let bottom_panel = build_bottom_panel(state);

    column![
        row![
            language_selector,
            column![].width(Length::Fill),
            run_button,
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        column![].height(8),
        code_editor,
        column![].height(12),
        bottom_panel,
    ]
    .spacing(0)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
