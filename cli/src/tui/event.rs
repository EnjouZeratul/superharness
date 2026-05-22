//! TUI 事件处理

use crossterm::event::{Event, KeyEvent};

#[allow(dead_code)]
pub enum AppEvent {
    Key(KeyEvent),
    Resize(u16, u16),
    Tick,
}

#[allow(dead_code)]
pub fn handle_event(event: Event) -> Option<AppEvent> {
    match event {
        Event::Key(key) => Some(AppEvent::Key(key)),
        Event::Resize(w, h) => Some(AppEvent::Resize(w, h)),
        _ => None,
    }
}
