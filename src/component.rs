// SPDX-FileCopyrightText: Copyright © 2024 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Component APIs

use bitflags::bitflags;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{layout::Rect, Frame};

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Quit,
    Redraw,
    Noop,
}

#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct State : u8 {
        const NONE = 1;
        const HIGHLIGHT = 1 << 1;
        const ACTIVE = 1 << 2;
    }
}

pub trait Component {
    fn render(&self, frame: &mut Frame, area: Rect);
    fn update(&mut self, action: Action) -> Option<Action>;

    // State management funcs
    fn state(&self) -> State;
    fn push_state(&mut self, st: State);
    fn pop_state(&mut self, st: State);
}
