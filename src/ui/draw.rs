use crate::ui::{App, MenuState, WarningState};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Chart, Dataset, Gauge, Paragraph, Wrap},
    Frame,
};

pub fn render(app: &App, frame: &mut Frame) -> anyhow::Result<()> {
    let min_width = 82;
    let min_height = 22;

    let current_size = frame.size();

    if current_size.width < min_width || current_size.height < min_height {
        let message = format!(
            "Terminal too small\nMinimum size: {}x{}\nCurrent size: {}x{}",
            min_width, min_height, current_size.width, current_size.height
        );
        let paragraph = Paragraph::new(message)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
        frame.render_widget(paragraph, current_size);
        return Ok(());
    }

    let stats_height = if current_size.height < 15 { 3 } else { 6 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(stats_height)])
        .split(frame.size());

    if app.warning_state != WarningState::None {
        draw_warning(app, frame, chunks[0]);
    } else if app.menu_state == MenuState::TestComplete {
        draw_test_complete(app, frame, chunks[0]);
    } else if app.menu_state != MenuState::Typing {
        draw_menu(app, frame, chunks[0]);
    } else {
        draw_typing_area(app, frame, chunks[0]);
    }

    draw_stats(app, frame, chunks[1]);

    Ok(())
}

fn draw_typing_area(app: &App, frame: &mut Frame, area: Rect) {
    if area.width < 30 || area.height < 5 {
        let text = "Terminal too small";
        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White));
        frame.render_widget(paragraph, area);
        return;
    }

    let app_title = format!(
        "TuiType{}",
        if app.config.repeat_test {
            " [Repeat Mode]"
        } else {
            ""
        }
    );

    let test_mode_str = match app.config.test_mode {
        crate::config::TestMode::Timed(secs) => format!("Mode: Timed {}s", secs),
        crate::config::TestMode::Words(count) => format!("Mode: Words {}", count),
        crate::config::TestMode::Quote => "Mode: Quote".to_string(),
        crate::config::TestMode::Custom => "Mode: Custom".to_string(),
    };

    let diff_str = match app.config.difficulty {
        crate::config::Difficulty::Easy => "Difficulty: Easy",
        crate::config::Difficulty::Medium => "Difficulty: Medium",
        crate::config::Difficulty::Hard => "Difficulty: Hard",
        crate::config::Difficulty::Custom => "Difficulty: Custom",
    };

    let repeat_mode_str = format!(
        "Repeat: {}",
        if app.config.repeat_test { "ON" } else { "OFF" }
    );

    let end_on_error_str = format!(
        "End on Error: {}",
        if app.config.end_on_first_error {
            "Yes"
        } else {
            "No"
        }
    );

    let time_remaining_str = if let Some(remaining) = app.time_remaining {
        match app.config.test_mode {
            crate::config::TestMode::Timed(_) => format!("Time: {}s", remaining),
            _ => String::new(),
        }
    } else {
        match app.config.test_mode {
            crate::config::TestMode::Timed(secs) => format!("Time: {}s", secs),
            _ => String::new(),
        }
    };

    let stats_str = format!(
        "WPM: {:.1} | Raw WPM: {:.1} | Acc: {:.1}%",
        app.stats.wpm, app.stats.raw_wpm, app.stats.accuracy
    );

    let single_line = if !time_remaining_str.is_empty() {
        format!(
            "{} | {} | {} | {} | {} | {} | {} | Press ESC for menu",
            app_title,
            test_mode_str,
            diff_str,
            repeat_mode_str,
            end_on_error_str,
            stats_str,
            time_remaining_str
        )
    } else {
        format!(
            "{} | {} | {} | {} | {} | {} | Press ESC for menu",
            app_title, test_mode_str, diff_str, repeat_mode_str, end_on_error_str, stats_str
        )
    };

    let width_available = (area.width as f32 * 0.9) as usize;
    let use_single_line = single_line.chars().count() <= width_available;

    let (block, typing_area) = if use_single_line {
        let block = Block::default()
            .title(single_line)
            .title_alignment(Alignment::Left)
            .title_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let inner_area = block.inner(area);
        (block, inner_area)
    } else {
        let first_line;
        let second_line;

        if area.width < 40 {
            let esc_menu_text = "ESC:Menu";
            let show_time = !time_remaining_str.is_empty();

            if show_time {
                first_line = format!("{} | {}", app_title, time_remaining_str);
                second_line = format!("WPM: {:.1} | {}", app.stats.wpm, esc_menu_text);
            } else {
                first_line = format!("{}", app_title);
                second_line = format!("WPM: {:.1} | {}", app.stats.wpm, esc_menu_text);
            }
        } else if area.width < 60 {
            let esc_menu_text = "Press ESC for menu";
            let show_time = !time_remaining_str.is_empty();

            if show_time {
                first_line = format!("{} | {}", app_title, time_remaining_str);
            } else {
                first_line = format!("{} | {}", app_title, test_mode_str);
            }

            second_line = format!(
                "WPM: {:.1} | Raw: {:.1} | {}",
                app.stats.wpm, app.stats.raw_wpm, esc_menu_text
            );
        } else {
            let first_row_with_config = if area.width <= 90 {
                format!("{} | {}", app_title, test_mode_str)
            } else {
                format!(
                    "{} | {} | {} | {} | {}",
                    app_title, test_mode_str, diff_str, repeat_mode_str, end_on_error_str
                )
            };

            let first_row_with_time = if !time_remaining_str.is_empty() {
                format!("{} | {}", first_row_with_config, time_remaining_str)
            } else {
                first_row_with_config.clone()
            };

            if area.width <= 90 {
                first_line = format!("{} | {}", first_row_with_time, stats_str);
                second_line = format!(
                    "{} | {} | {} | Press ESC for menu",
                    diff_str, repeat_mode_str, end_on_error_str
                );
            } else if first_row_with_time.chars().count() + stats_str.chars().count() + 3
                <= width_available
            {
                first_line = format!("{} | {}", first_row_with_time, stats_str);
                second_line = format!("Press ESC for menu");
            } else if first_row_with_config.chars().count() + time_remaining_str.chars().count() + 3
                <= width_available
            {
                first_line = first_row_with_time;
                second_line = format!("{} | Press ESC for menu", stats_str);
            } else {
                first_line = first_row_with_config;
                second_line = format!("{} | Press ESC for menu", stats_str);
            }
        }

        let block = Block::default()
            .title(first_line)
            .title_alignment(Alignment::Left)
            .title_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let inner_area = block.inner(area);

        if inner_area.height > 1 {
            let settings_block = Block::default()
                .title(second_line.clone())
                .title_alignment(Alignment::Left)
                .title_style(
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )
                .borders(Borders::NONE);

            let line2_area = Rect::new(inner_area.x, inner_area.y, inner_area.width, 1);
            frame.render_widget(settings_block, line2_area);

            if area.width < 80 && inner_area.height > 2 && area.width >= 50 {
                let third_line = format!("{}", stats_str);

                let stats_block = Block::default()
                    .title(third_line)
                    .title_alignment(Alignment::Left)
                    .title_style(
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )
                    .borders(Borders::NONE);

                let line3_area = Rect::new(inner_area.x, inner_area.y + 1, inner_area.width, 1);
                frame.render_widget(stats_block, line3_area);

                let typing_area = Rect::new(
                    inner_area.x,
                    inner_area.y + 2,
                    inner_area.width,
                    inner_area.height.saturating_sub(2),
                );

                (block, typing_area)
            } else {
                let typing_area = Rect::new(
                    inner_area.x,
                    inner_area.y + 1,
                    inner_area.width,
                    inner_area.height.saturating_sub(1),
                );

                (block, typing_area)
            }
        } else {
            (block, inner_area)
        }
    };

    frame.render_widget(block, area);

    if typing_area.height > 0 {
        render_typing_text(app, frame, typing_area);
    }
}

fn render_typing_text(app: &App, frame: &mut Frame, typing_area: Rect) {
    let target_text = app.text_source.full_text();

    let is_quote_mode = matches!(app.config.test_mode, crate::config::TestMode::Quote);
    let display_window_size = typing_area.width as usize * 4;

    let (start_pos, target_display_text) =
        if is_quote_mode && target_text.len() > display_window_size {
            let visible_start = if app.cursor_pos > display_window_size / 2 {
                let ideal_start = app.cursor_pos - display_window_size / 2;
                if ideal_start > 0 {
                    match target_text[..ideal_start].rfind(' ') {
                        Some(pos) => pos + 1,
                        None => 0,
                    }
                } else {
                    0
                }
            } else {
                0
            };

            let visible_end = (visible_start + display_window_size).min(target_text.len());
            (visible_start, &target_text[visible_start..visible_end])
        } else {
            (0, target_text)
        };

    let mut styled_spans = Vec::new();

    let correct_style = Style::default().fg(Color::Rgb(
        app.theme.correct.0,
        app.theme.correct.1,
        app.theme.correct.2,
    ));

    let incorrect_style = Style::default().fg(Color::Rgb(
        app.theme.incorrect.0,
        app.theme.incorrect.1,
        app.theme.incorrect.2,
    ));

    let pending_style = Style::default().fg(Color::Rgb(
        app.theme.pending.0,
        app.theme.pending.1,
        app.theme.pending.2,
    ));

    for (i, ch) in target_display_text.chars().enumerate() {
        let absolute_pos = start_pos + i;
        let span = if absolute_pos < app.typed_text.len() {
            let typed_char = app.typed_text.chars().nth(absolute_pos).unwrap();
            if typed_char == ch {
                Span::styled(ch.to_string(), correct_style)
            } else {
                Span::styled(ch.to_string(), incorrect_style)
            }
        } else if absolute_pos == app.cursor_pos {
            Span::styled(
                ch.to_string(),
                Style::default()
                    .fg(Color::Rgb(
                        app.theme.cursor.0,
                        app.theme.cursor.1,
                        app.theme.cursor.2,
                    ))
                    .add_modifier(Modifier::REVERSED),
            )
        } else {
            Span::styled(ch.to_string(), pending_style)
        };

        styled_spans.push(span);
    }

    if app.typed_text.len() > target_text.len() {
        for (i, ch) in app.typed_text.chars().skip(target_text.len()).enumerate() {
            let pos = target_text.len() + i;
            let span = if pos == app.cursor_pos {
                Span::styled(
                    ch.to_string(),
                    Style::default()
                        .fg(Color::Rgb(
                            app.theme.incorrect.0,
                            app.theme.incorrect.1,
                            app.theme.incorrect.2,
                        ))
                        .add_modifier(Modifier::REVERSED),
                )
            } else {
                Span::styled(ch.to_string(), incorrect_style)
            };
            styled_spans.push(span);
        }
    }

    if app.cursor_pos >= app.typed_text.len() && app.cursor_pos >= target_text.len() {
        styled_spans.push(Span::styled(
            " ",
            Style::default()
                .fg(Color::Rgb(
                    app.theme.cursor.0,
                    app.theme.cursor.1,
                    app.theme.cursor.2,
                ))
                .add_modifier(Modifier::REVERSED),
        ));
    }

    let text = Text::from(Line::from(styled_spans));

    let paragraph = Paragraph::new(text)
        .block(Block::default())
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Left);

    frame.render_widget(paragraph, typing_area);
}

fn draw_test_complete_new(app: &App, frame: &mut Frame, area: Rect) {
    let app_title = format!(
        "TuiType{}",
        if app.config.repeat_test {
            " [Repeat Mode]"
        } else {
            ""
        }
    );

    let width = area
        .width
        .saturating_sub(10)
        .min(60)
        .max(20)
        .min(area.width);
    let height = area
        .height
        .saturating_sub(2)
        .min(15)
        .max(3)
        .min(area.height);

    if width < 15 || height < 3 {
        let text = "Test Complete\nPress ENTER to restart";
        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White));
        frame.render_widget(paragraph, area);
        return;
    }

    if height < 5 || width < 25 {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Test Complete ")
            .title_style(Style::default().fg(Color::White));

        frame.render_widget(block.clone(), area);
        let inner_area = block.inner(area);

        let results = format!(
            "WPM: {:.1} | Raw WPM: {:.1} | Acc: {:.1}%",
            app.stats.wpm, app.stats.raw_wpm, app.stats.accuracy
        );
        let paragraph = Paragraph::new(vec![
            Line::from(vec![Span::styled(
                results,
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from("Press ENTER to restart"),
        ])
        .alignment(Alignment::Center);

        frame.render_widget(paragraph, inner_area);
        return;
    }

    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;

    let popup_area = Rect::new(x, y, width, height);

    let background = Block::default().style(Style::default().bg(Color::Black));
    frame.render_widget(background, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} - TEST COMPLETE ", app_title))
        .title_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(Color::White));

    frame.render_widget(block.clone(), popup_area);

    let duration = if let (Some(start), Some(end)) = (app.start_time, app.end_time) {
        end.duration_since(start).as_secs_f64()
    } else {
        0.0
    };

    let inner_area = block.inner(popup_area);

    let has_two_columns = inner_area.width >= 40 && inner_area.height >= 8;

    let columns = if has_two_columns {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(inner_area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)])
            .split(inner_area)
    };

    let mut results_lines = if inner_area.height < 8 {
        vec![Line::from(vec![Span::styled(
            format!(
                "WPM: {:.1} | Raw WPM: {:.1} | Acc: {:.1}%",
                app.stats.wpm, app.stats.raw_wpm, app.stats.accuracy
            ),
            Style::default().add_modifier(Modifier::BOLD),
        )])]
    } else {
        vec![
            Line::from(vec![Span::styled(
                "TEST RESULTS",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::default(),
            Line::from(vec![
                Span::raw("WPM: "),
                Span::styled(
                    format!("{:.1}", app.stats.wpm),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("Raw WPM: "),
                Span::styled(
                    format!("{:.1}", app.stats.raw_wpm),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("Accuracy: "),
                Span::styled(
                    format!("{:.1}%", app.stats.accuracy),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("Time: "),
                Span::styled(
                    format!("{:.1}s", duration),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ]),
        ]
    };

    if let Some(reason) = &app.test_end_reason {
        results_lines.push(Line::default());
        results_lines.push(Line::from(vec![
            Span::raw("Note: "),
            Span::styled(
                reason,
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    let test_mode_str = match app.config.test_mode {
        crate::config::TestMode::Timed(secs) => format!("Timed - {}s", secs),
        crate::config::TestMode::Words(count) => format!("Words - {}", count),
        crate::config::TestMode::Quote => "Quote".to_string(),
        crate::config::TestMode::Custom => "Custom".to_string(),
    };

    let diff_str = match app.config.difficulty {
        crate::config::Difficulty::Easy => "Easy",
        crate::config::Difficulty::Medium => "Medium",
        crate::config::Difficulty::Hard => "Hard",
        crate::config::Difficulty::Custom => "Custom",
    };

    let settings_lines = vec![
        Line::from(vec![Span::styled(
            "TEST SETTINGS",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::default(),
        Line::from(vec![
            Span::raw("Mode: "),
            Span::styled(
                &test_mode_str,
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("Difficulty: "),
            Span::styled(diff_str, Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw("Repeat Mode: "),
            Span::styled(
                if app.config.repeat_test { "ON" } else { "OFF" },
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("End on First Error: "),
            Span::styled(
                if app.config.end_on_first_error {
                    "ON"
                } else {
                    "OFF"
                },
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::default(),
    ];

    let divider = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(Color::White));

    if has_two_columns {
        frame.render_widget(divider, columns[1]);
    }

    let total_height = inner_area.height;

    if !has_two_columns {
        let mut combined_lines = Vec::new();

        if total_height < 8 {
            combined_lines.push(Line::from(vec![Span::styled(
                format!(
                    "WPM: {:.1} | Raw WPM: {:.1} | Acc: {:.1}%",
                    app.stats.wpm, app.stats.raw_wpm, app.stats.accuracy
                ),
                Style::default().add_modifier(Modifier::BOLD),
            )]));

            if let Some(reason) = &app.test_end_reason {
                combined_lines.push(Line::from(vec![Span::styled(
                    reason,
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )]));
            }

            combined_lines.push(Line::from(vec![Span::styled(
                "Press ENTER to restart",
                Style::default().fg(Color::Rgb(
                    app.theme.text.0,
                    app.theme.text.1,
                    app.theme.text.2,
                )),
            )]));
        } else {
            combined_lines = results_lines.clone();
            combined_lines.push(Line::default());
            combined_lines.push(Line::default());
            combined_lines.extend_from_slice(&settings_lines);
        }

        let content_height = combined_lines.len() as u16;
        let padding_top = if total_height > content_height {
            (total_height - content_height) / 2
        } else {
            0
        };

        let content_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(padding_top),
                Constraint::Min(content_height),
                Constraint::Min(1),
            ])
            .split(columns[0])[1];

        let combined_paragraph = Paragraph::new(combined_lines)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White));

        frame.render_widget(combined_paragraph, content_area);

        if total_height >= 8 {
            let note_y = (columns[0].y + columns[0].height).saturating_sub(2);
            if note_y > columns[0].y {
                let note_area = Rect::new(columns[0].x, note_y, columns[0].width, 1);

                let restart_note = Line::from(vec![Span::styled(
                    "Press ENTER to restart typing test",
                    Style::default().fg(Color::Rgb(
                        app.theme.text.0,
                        app.theme.text.1,
                        app.theme.text.2,
                    )),
                )])
                .alignment(Alignment::Center);

                let note_paragraph = Paragraph::new(vec![restart_note])
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::White));

                frame.render_widget(note_paragraph, note_area);
            }
        }
    } else {
        let content_height = results_lines.len().max(settings_lines.len()) as u16;
        let padding_top = if total_height > content_height {
            (total_height - content_height) / 2
        } else {
            0
        };

        let left_column = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(padding_top),
                Constraint::Min(content_height),
                Constraint::Min(1),
            ])
            .split(columns[0])[1];

        let right_column = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(padding_top),
                Constraint::Min(content_height),
                Constraint::Min(1),
            ])
            .split(columns[1])[1];

        let results_paragraph = Paragraph::new(results_lines)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White));
        frame.render_widget(results_paragraph, left_column);

        let settings_paragraph = Paragraph::new(settings_lines)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White));
        frame.render_widget(settings_paragraph, right_column);

        if columns[0].height > content_height + padding_top + 2 {
            let restart_note = Line::from(vec![Span::styled(
                "Press ENTER to restart typing test",
                Style::default().fg(Color::Rgb(
                    app.theme.text.0,
                    app.theme.text.1,
                    app.theme.text.2,
                )),
            )])
            .alignment(Alignment::Center);

            let note_area = Rect::new(
                columns[0].x,
                columns[0].y + columns[0].height - 2,
                columns[0].width,
                1,
            );

            let note_paragraph = Paragraph::new(vec![restart_note])
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::White));

            frame.render_widget(note_paragraph, note_area);
        }
    }
}

fn draw_stats(app: &App, frame: &mut Frame, area: Rect) {
    if area.width < 8 || area.height < 2 {
        return;
    }

    if area.width < 30 || area.height < 3 {
        draw_compact_stats(app, frame, area);
        return;
    }

    if area.width < 40 || area.height < 5 {
        draw_minimal_gauge(app, frame, area);
        return;
    }

    if area.width < 60 || area.height < 6 {
        draw_gauges(app, frame, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(25), Constraint::Min(30)])
        .split(area);

    draw_gauges(app, frame, chunks[0]);
    draw_chart(app, frame, chunks[1]);
}

fn draw_compact_stats(app: &App, frame: &mut Frame, area: Rect) {
    let label = format!(
        "WPM: {:.0} Raw WPM: {:.0} Acc: {:.0}%",
        app.stats.wpm, app.stats.raw_wpm, app.stats.accuracy
    );
    let paragraph = Paragraph::new(label)
        .style(Style::default().fg(Color::Rgb(
            app.theme.correct.0,
            app.theme.correct.1,
            app.theme.correct.2,
        )))
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

fn draw_minimal_gauge(app: &App, frame: &mut Frame, area: Rect) {
    let accuracy_value = (app.stats.accuracy.clamp(0.0, 100.0) as u16).min(100);
    let accuracy_label = format!("{:.1}%", app.stats.accuracy);

    let accuracy_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Acc"))
        .gauge_style(Style::default().fg(Color::Rgb(
            app.theme.correct.0,
            app.theme.correct.1,
            app.theme.correct.2,
        )))
        .percent(accuracy_value)
        .label(accuracy_label);

    frame.render_widget(accuracy_gauge, area);
}

fn draw_gauges(app: &App, frame: &mut Frame, area: Rect) {
    if area.width < 10 || area.height < 3 {
        return;
    }

    let accuracy_value = (app.stats.accuracy.clamp(0.0, 100.0) as u16).min(100);

    if area.height < 5 {
        let accuracy_label = format!("Accuracy: {:.1}%", app.stats.accuracy);
        let accuracy_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Accuracy"))
            .gauge_style(Style::default().fg(Color::Rgb(
                app.theme.correct.0,
                app.theme.correct.1,
                app.theme.correct.2,
            )))
            .percent(accuracy_value)
            .label(accuracy_label);
        frame.render_widget(accuracy_gauge, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3)])
        .split(area);

    let accuracy_label = format!("Accuracy: {:.1}%", app.stats.accuracy);
    let accuracy_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Accuracy"))
        .gauge_style(Style::default().fg(Color::Rgb(
            app.theme.correct.0,
            app.theme.correct.1,
            app.theme.correct.2,
        )))
        .percent(accuracy_value)
        .label(accuracy_label);
    frame.render_widget(accuracy_gauge, chunks[0]);

    let progress = if app.text_source.full_text().is_empty() {
        0
    } else if app.text_source.is_scrollable {
        let total_words = app.text_source.total_words() as usize;
        if total_words == 0 {
            0
        } else {
            let typed_words = app.typed_text.split_whitespace().count();
            ((typed_words * 100) / total_words) as u16
        }
    } else {
        ((app.typed_text.len().min(app.text_source.full_text().len()) * 100)
            / app.text_source.full_text().len()) as u16
    };

    let progress_value = progress.min(100);

    let progress_label = format!("Progress: {}%", progress_value);
    let progress_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Progress"))
        .gauge_style(Style::default().fg(Color::Rgb(
            app.theme.pending.0,
            app.theme.pending.1,
            app.theme.pending.2,
        )))
        .percent(progress_value)
        .label(progress_label);
    frame.render_widget(progress_gauge, chunks[1]);
}

fn draw_chart(app: &App, frame: &mut Frame, area: Rect) {
    if area.width < 20 || area.height < 4 {
        if !app.stats.wpm_samples.is_empty() {
            let latest_wpm = app.stats.wpm_samples.last().unwrap_or(&0.0);
            let placeholder = format!("WPM: {:.1}", latest_wpm);
            let placeholder_widget = Paragraph::new(placeholder)
                .block(Block::default().borders(Borders::ALL).title("Current WPM"))
                .alignment(Alignment::Center);
            frame.render_widget(placeholder_widget, area);
        }
        return;
    }

    let effective_samples = if app.stats.wpm_samples.is_empty() {
        vec![0.0]
    } else {
        app.stats.wpm_samples.clone()
    };

    let raw_wpm_samples = if app.stats.raw_wpm_samples.is_empty() {
        vec![0.0]
    } else {
        app.stats.raw_wpm_samples.clone()
    };

    let wpm_data: Vec<(f64, f64)> = effective_samples
        .iter()
        .enumerate()
        .map(|(i, &wpm)| (i as f64, wpm))
        .collect();

    let raw_wpm_data: Vec<(f64, f64)> = raw_wpm_samples
        .iter()
        .enumerate()
        .map(|(i, &wpm)| (i as f64, wpm))
        .collect();

    let mut max_wpm = 20.0f64;
    for &wpm in &effective_samples {
        if wpm > max_wpm {
            max_wpm = wpm;
        }
    }

    for &raw_wpm in &raw_wpm_samples {
        if raw_wpm > max_wpm {
            max_wpm = raw_wpm;
        }
    }

    max_wpm = max_wpm.max(20.0) * 1.1;

    let datasets = vec![
        Dataset::default()
            .name("WPM")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Rgb(
                app.theme.accent.0,
                app.theme.accent.1,
                app.theme.accent.2,
            )))
            .data(&wpm_data),
        Dataset::default()
            .name("Raw WPM")
            .marker(symbols::Marker::Dot)
            .style(Style::default().fg(Color::Rgb(
                app.theme.incorrect.0,
                app.theme.incorrect.1,
                app.theme.incorrect.2,
            )))
            .data(&raw_wpm_data),
    ];

    let sample_count = effective_samples.len().max(1) as f64;

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title("WPM Over Time")
                .borders(Borders::ALL),
        )
        .x_axis(
            ratatui::widgets::Axis::default()
                .title("Time")
                .style(Style::default().fg(Color::Rgb(
                    app.theme.text.0,
                    app.theme.text.1,
                    app.theme.text.2,
                )))
                .bounds([0.0, sample_count.max(1.0)])
                .labels(vec![
                    Span::raw("0"),
                    Span::raw(format!("{}", sample_count as usize)),
                ]),
        )
        .y_axis(
            ratatui::widgets::Axis::default()
                .title("WPM")
                .style(Style::default().fg(Color::Rgb(
                    app.theme.text.0,
                    app.theme.text.1,
                    app.theme.text.2,
                )))
                .bounds([0.0, max_wpm])
                .labels(vec![
                    Span::raw("0"),
                    Span::raw(format!("{:.0}", max_wpm / 2.0)),
                    Span::raw(format!("{:.0}", max_wpm)),
                ]),
        );

    frame.render_widget(chart, area);
}

fn draw_menu(app: &App, frame: &mut Frame, area: Rect) {
    let app_title = format!(
        "TuiType{}",
        if app.config.repeat_test {
            " [Repeat Mode]"
        } else {
            ""
        }
    );

    let width = area.width.saturating_sub(4).min(area.width);
    let height = area.height.saturating_sub(2).min(area.height);

    if width < 30 || height < 10 {
        let menu_type = match app.menu_state {
            MenuState::MainMenu(_) => "Main Menu",
            MenuState::TestModeMenu(_) => "Test Mode",
            MenuState::DifficultyMenu(_) => "Difficulty",
            MenuState::TimeMenu(_) => "Time Limit",
            MenuState::WordCountMenu(_) => "Word Count",
            MenuState::ThemeMenu(_) => "Theme",
            MenuState::Help => "Help",
            _ => "Menu",
        };

        let text = format!("{}\nPress ESC to return", menu_type);
        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White));
        frame.render_widget(paragraph, area);
        return;
    }

    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let menu_area = Rect::new(x, y, width, height);

    let menu_type = match app.menu_state {
        MenuState::MainMenu(_) => "MAIN MENU",
        MenuState::TestModeMenu(_) => "TEST MODE",
        MenuState::DifficultyMenu(_) => "DIFFICULTY",
        MenuState::TimeMenu(_) => "TIME LIMIT",
        MenuState::WordCountMenu(_) => "WORD COUNT",
        MenuState::ThemeMenu(_) => "THEME",
        MenuState::CustomTimedInput(_) => "CUSTOM TIMED TEST",
        MenuState::CustomWordsInput(_) => "CUSTOM WORDS TEST",
        MenuState::SettingsMenu(_) => "SETTINGS",
        MenuState::Help => "HELP",
        MenuState::TestComplete => "TEST COMPLETE",
        _ => "",
    };

    let title = format!(" {} - {} ", app_title, menu_type);

    let outline = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(Color::White));

    frame.render_widget(outline.clone(), menu_area);

    let inner_area = outline.inner(menu_area);

    let menu_text = match app.menu_state {
        MenuState::MainMenu(idx) => {
            let mut text = Vec::new();
            let items = vec![
                ("1. Test Mode", idx == 0),
                ("2. Difficulty", idx == 1),
                ("3. Theme", idx == 2),
                ("4. Settings", idx == 3),
                ("5. Help", idx == 4),
                ("6. Back", idx == 5),
            ];

            for (item, selected) in items {
                if selected {
                    text.push(Line::from(vec![Span::styled(
                        format!("> {} <", item),
                        Style::default().add_modifier(Modifier::REVERSED),
                    )]));
                } else {
                    text.push(Line::from(item));
                }
            }

            text
        }
        MenuState::TestModeMenu(idx) => {
            let items = [
                ("1. Timed", idx == 0),
                ("2. Words", idx == 1),
                ("3. Quote", idx == 2),
                ("4. Back", idx == 3),
            ];

            items
                .iter()
                .map(|(item, selected)| {
                    if *selected {
                        Line::from(vec![Span::styled(
                            format!("> {} <", item),
                            Style::default().add_modifier(Modifier::REVERSED),
                        )])
                    } else {
                        Line::from(*item)
                    }
                })
                .collect()
        }
        MenuState::DifficultyMenu(idx) => {
            let items = [
                ("1. Easy", idx == 0),
                ("2. Medium", idx == 1),
                ("3. Hard", idx == 2),
                ("4. Back", idx == 3),
            ];

            items
                .iter()
                .map(|(item, selected)| {
                    if *selected {
                        Line::from(vec![Span::styled(
                            format!("> {} <", item),
                            Style::default().add_modifier(Modifier::REVERSED),
                        )])
                    } else {
                        Line::from(*item)
                    }
                })
                .collect()
        }
        MenuState::TimeMenu(idx) => {
            let items = [
                ("1. 15 seconds", idx == 0),
                ("2. 30 seconds", idx == 1),
                ("3. 60 seconds", idx == 2),
                ("4. 120 seconds", idx == 3),
                ("5. Custom...", idx == 4),
                ("6. Back", idx == 5),
            ];

            items
                .iter()
                .map(|(item, selected)| {
                    if *selected {
                        Line::from(vec![Span::styled(
                            format!("> {} <", item),
                            Style::default().add_modifier(Modifier::REVERSED),
                        )])
                    } else {
                        Line::from(*item)
                    }
                })
                .collect()
        }
        MenuState::WordCountMenu(idx) => {
            let items = [
                ("1. 10 words", idx == 0),
                ("2. 25 words", idx == 1),
                ("3. 50 words", idx == 2),
                ("4. Custom...", idx == 3),
                ("5. Back", idx == 4),
            ];

            items
                .iter()
                .map(|(item, selected)| {
                    if *selected {
                        Line::from(vec![Span::styled(
                            format!("> {} <", item),
                            Style::default().add_modifier(Modifier::REVERSED),
                        )])
                    } else {
                        Line::from(*item)
                    }
                })
                .collect()
        }
        MenuState::ThemeMenu(idx) => {
            let items = [
                ("1. Light", idx == 0),
                ("2. Dark", idx == 1),
                ("3. Sepia", idx == 2),
                ("4. Matrix", idx == 3),
                ("5. Ocean", idx == 4),
                ("6. Back", idx == 5),
            ];

            items
                .iter()
                .map(|(item, selected)| {
                    if *selected {
                        Line::from(vec![Span::styled(
                            format!("> {} <", item),
                            Style::default().add_modifier(Modifier::REVERSED),
                        )])
                    } else {
                        Line::from(*item)
                    }
                })
                .collect()
        }
        MenuState::SettingsMenu(idx) => {
            let items = [
                ("1. Toggle Repeat Mode", idx == 0),
                ("2. Toggle End on First Error", idx == 1),
                ("3. Back", idx == 2),
            ];
            vec![
                if items[0].1 {
                    Line::from(vec![Span::styled(
                        format!("> {} <", items[0].0),
                        Style::default().add_modifier(Modifier::REVERSED),
                    )])
                } else {
                    Line::from(items[0].0)
                },
                Line::default(),
                Line::from(format!(
                    "Current: {}",
                    if app.config.repeat_test { "ON" } else { "OFF" }
                )),
                Line::default(),
                if items[1].1 {
                    Line::from(vec![Span::styled(
                        format!("> {} <", items[1].0),
                        Style::default().add_modifier(Modifier::REVERSED),
                    )])
                } else {
                    Line::from(items[1].0)
                },
                Line::default(),
                Line::from(format!(
                    "Current: {}",
                    if app.config.end_on_first_error {
                        "ON"
                    } else {
                        "OFF"
                    }
                )),
                Line::default(),
                if items[2].1 {
                    Line::from(vec![Span::styled(
                        format!("> {} <", items[2].0),
                        Style::default().add_modifier(Modifier::REVERSED),
                    )])
                } else {
                    Line::from(items[2].0)
                },
            ]
        }

        MenuState::CustomTimedInput(ref input) => {
            vec![
                Line::from(Span::styled(
                    "ENTER CUSTOM TIME (SECONDS):",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::default(),
                Line::from(if input.is_empty() {
                    vec![Span::styled(
                        "▋",
                        Style::default().add_modifier(Modifier::SLOW_BLINK),
                    )]
                } else {
                    vec![Span::styled(
                        format!("{} ▋", input),
                        Style::default().add_modifier(Modifier::BOLD),
                    )]
                }),
                Line::default(),
                Line::from("Press ENTER to confirm"),
            ]
        }

        MenuState::CustomWordsInput(ref input) => {
            vec![
                Line::from(Span::styled(
                    "ENTER CUSTOM WORD COUNT:",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::default(),
                Line::from(if input.is_empty() {
                    vec![Span::styled(
                        "▋",
                        Style::default().add_modifier(Modifier::SLOW_BLINK),
                    )]
                } else {
                    vec![Span::styled(
                        format!("{} ▋", input),
                        Style::default().add_modifier(Modifier::BOLD),
                    )]
                }),
                Line::default(),
                Line::from("Press ENTER to confirm"),
            ]
        }
        MenuState::Help => {
            vec![
                Line::from(vec![Span::styled(
                    "KEYBOARD CONTROLS",
                    Style::default().add_modifier(Modifier::BOLD),
                )]),
                Line::from("• Esc: Open menu / Close menu / Cancel current test"),
                Line::from("• Tab: Quick restart test"),
                Line::from("• Ctrl+C: Exit application"),
                Line::from("• ↑/↓: Navigate menus or scroll help"),
                Line::from("• Enter: Select menu option"),
                Line::default(),
                Line::from(vec![Span::styled(
                    "TEST MODES",
                    Style::default().add_modifier(Modifier::BOLD),
                )]),
                Line::from("• Timed: Type as many words as possible within time limit"),
                Line::from("• Words: Type a specific number of words"),
                Line::from("• Quote: Type a random quote"),
                Line::from("• Custom: Type custom text (set in config file)"),
                Line::default(),
                Line::from(vec![Span::styled(
                    "SETTINGS",
                    Style::default().add_modifier(Modifier::BOLD),
                )]),
                Line::from("• Repeat Mode: Practice the same text multiple times"),
                Line::from("  - Perfect for practicing problematic words"),
                Line::from("  - Settings cannot be changed while active"),
                Line::from("• End on First Error: Test stops on first mistake"),
                Line::from("  - Useful for perfect accuracy practice"),
                Line::default(),
                Line::from(vec![Span::styled(
                    "STATISTICS",
                    Style::default().add_modifier(Modifier::BOLD),
                )]),
                Line::from("• WPM (Words Per Minute): Based on 5 characters = 1 word"),
                Line::from("• Raw WPM: Speed without error penalty"),
                Line::from("• Accuracy: Percentage of correct characters"),
                Line::default(),
                Line::from(vec![Span::styled(
                    "THEMES",
                    Style::default().add_modifier(Modifier::BOLD),
                )]),
                Line::from("• Light: High contrast light theme"),
                Line::from("• Dark: High contrast dark theme"),
                Line::from("• Sepia: Easy on the eyes, warm colors"),
                Line::from("• Matrix: Classic green on black"),
                Line::from("• Ocean: Calming blue tones"),
                Line::default(),
                Line::from(vec![Span::styled(
                    "CREDITS",
                    Style::default().add_modifier(Modifier::BOLD),
                )]),
                Line::from("TuiType - A Rust-based typing test application"),
                Line::from("Created by RobbyV2 - github.com/RobbyV2/tuitype"),
                Line::from("Licensed under MIT License"),
                Line::from(format!("Version {}", crate::VERSION)),
                Line::default(),
                Line::from("Use ↑/↓ to scroll, Esc to return to menu"),
            ]
        }
        MenuState::TestComplete => {
            let duration = if let (Some(start), Some(end)) = (app.start_time, app.end_time) {
                end.duration_since(start).as_secs_f64()
            } else {
                0.0
            };

            let mut lines = vec![
                Line::from(Span::styled(
                    "TEST RESULTS",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::default(),
                Line::from(format!("WPM: {:.1}", app.stats.wpm)),
                Line::from(format!("Raw WPM: {:.1}", app.stats.raw_wpm)),
                Line::from(format!("Accuracy: {:.1}%", app.stats.accuracy)),
                Line::from(format!("Time: {:.1} seconds", duration)),
            ];

            if let Some(reason) = &app.test_end_reason {
                lines.push(Line::default());
                lines.push(Line::from(Span::styled(
                    reason,
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )));
            }

            lines.push(Line::default());
            lines.push(Line::from("Press ENTER to restart test"));
            lines.push(Line::from("Press ESC to return to menu"));

            lines
        }

        _ => vec![Line::from("Press ESC to return to typing")],
    };

    let mut full_text = menu_text;

    if app.menu_state != MenuState::Help {
        full_text.push(Line::default());
        full_text.push(Line::from("UP/DOWN: Navigate    ENTER: Select"));
        full_text.push(Line::from("ESC: Return to typing test"));
    } else {
        full_text.push(Line::default());
        full_text.push(Line::from("Press ESC to return"));
    }

    let menu_paragraph = if app.menu_state == MenuState::Help {
        Paragraph::new(full_text)
            .alignment(Alignment::Left)
            .style(Style::default().fg(Color::White))
            .scroll((app.help_scroll_offset as u16, 0))
            .wrap(Wrap { trim: true })
    } else {
        Paragraph::new(full_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White))
    };

    frame.render_widget(menu_paragraph, inner_area);
}

fn draw_test_complete(app: &App, frame: &mut Frame, area: Rect) {
    draw_test_complete_new(app, frame, area);
}

fn draw_warning(app: &App, frame: &mut Frame, area: Rect) {
    let (action, _prev_state) = match &app.warning_state {
        WarningState::RepeatModeSettings { action, prev_state } => (action, prev_state),
        _ => return,
    };

    let app_title = format!(
        "TuiType{}",
        if app.config.repeat_test {
            " [Repeat Mode]"
        } else {
            ""
        }
    );

    let width = area
        .width
        .saturating_sub(10)
        .min(80)
        .max(30)
        .min(area.width);
    let height = 10.min(area.height.saturating_sub(4)).min(area.height);

    if width < 30 || height < 5 {
        let text = "Warning: Repeat Mode active\nPress ENTER to disable";
        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Red));
        frame.render_widget(paragraph, area);
        return;
    }

    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;

    let popup_area = Rect::new(x, y, width, height);

    let background = Block::default().style(Style::default().bg(Color::Black));
    frame.render_widget(background, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title(format!(" {} - REPEAT MODE WARNING ", app_title))
        .title_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(block.clone(), popup_area);

    let inner_area = block.inner(popup_area);

    let message_lines = if inner_area.height >= 8 {
        vec![
            Line::from(vec![Span::styled(
                "SETTINGS CHANGE RESTRICTED",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )]),
            Line::default(),
            Line::from(action.as_str()),
            Line::default(),
            Line::from("Changing settings during Repeat Mode would affect test consistency."),
            Line::default(),
            Line::from(vec![
                Span::styled(
                    "ENTER",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Disable Repeat Mode and continue"),
            ]),
            Line::from(vec![
                Span::styled(
                    "ESC",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Cancel and return to previous menu"),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![Span::styled(
                "SETTINGS RESTRICTED",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )]),
            Line::from(action.as_str()),
            Line::from("ENTER: Disable Repeat Mode"),
            Line::from("ESC: Cancel"),
        ]
    };

    let warning_paragraph = Paragraph::new(message_lines)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::White));

    frame.render_widget(warning_paragraph, inner_area);
}
