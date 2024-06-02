// SPDX-FileCopyrightText: Copyright © 2024 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! TextBox ...

use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Borders},
};
use tui_textarea::TextArea;

use crate::{component::State, Action, Component};

pub struct TextBox {
    area: TextArea<'static>,
    state: State,
}

impl TextBox {
    pub fn new(title: &str) -> Self {
        let mut text = TextArea::default();
        text.set_cursor_line_style(Style::default());
        text.set_style(Style::default());
        text.set_block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Gray))
                .title(title.to_string())
                .title_style(Style::default().fg(Color::Gray)),
        );

        Self {
            area: text,
            state: State::NONE,
        }
    }

    /// Set as a password field
    pub fn set_hide_chars(&mut self) {
        self.area.set_mask_char('•')
    }

    /// Update text style based on state
    fn style_from_state(&mut self) {
        let style = if self.state.contains(State::ACTIVE) {
            self.area.set_cursor_style(Style::default().reversed());
            Style::default().fg(Color::White)
        } else {
            self.area.set_cursor_style(Style::default());
            Style::default().fg(Color::White).dim()
        };
        let styled = if self.area.mask_char().is_some() {
            style.bold()
        } else {
            style
        };
        self.area.set_style(styled);
    }

    // Update block style based on state
    fn block_from_state(&mut self) {
        let block = if self.state.contains(State::ACTIVE) {
            self.area
                .block()
                .unwrap()
                .clone()
                .border_style(Style::default().fg(Color::LightBlue))
                .title_style(Style::default().fg(Color::Gray))
        } else if self.state.contains(State::HIGHLIGHT) {
            self.area
                .block()
                .unwrap()
                .clone()
                .border_style(Style::default().fg(Color::White))
                .title_style(Style::default().fg(Color::Gray))
        } else {
            self.area
                .block()
                .unwrap()
                .clone()
                .border_style(Style::default().fg(Color::Gray).dim())
                .title_style(Style::default().fg(Color::Gray).dim())
        };
        self.area.set_block(block);
    }
}

impl Component for TextBox {
    /// Render to bounds
    fn render(&self, frame: &mut ratatui::prelude::Frame, area: Rect) {
        frame.render_widget(self.area.widget(), area)
    }

    // Update state
    fn update(&mut self, action: crate::Action) -> Option<crate::Action> {
        match action {
            Action::Key(k) => self.area.input(k),
            Action::Mouse(m) => self.area.input(m),
            _ => false,
        };
        None
    }

    /// Return our state
    fn state(&self) -> State {
        self.state
    }

    /// Push a new state to the set
    fn push_state(&mut self, st: crate::component::State) {
        self.state.insert(st);
        self.block_from_state();
        self.style_from_state();
    }

    /// Pop a state from the set
    fn pop_state(&mut self, st: crate::component::State) {
        self.state.remove(st);
        self.block_from_state();
        self.style_from_state();
    }
}
