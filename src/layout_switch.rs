//! Host layout switching for the RU layer.
//!
//! The host toggles EN/RU on Caps Lock. RMK's macro `Tap` holds a key for only
//! 2 ms, and macOS ignores Caps Lock presses that short, so the switch was
//! randomly dropped or applied late. Here Caps Lock is held long enough and
//! given time to settle, with reports sent straight to the USB report channel.
//!
//! Each RU symbol key runs a tiny macro, `[TapAction(User(n)), Delay(..)]`
//! (see behavior.rs). The `Delay` keeps RMK's key processing paused while this
//! task types, so keys pressed in the meantime are queued and come out in order,
//! in the right layout.

use embassy_time::Timer;
use rmk::channel::USB_REPORT_CHANNEL;
use rmk::event::{ActionEvent, EventSubscriber, SubscribableEvent};
use rmk::hid::{KeyboardReport, Report};
use rmk::types::action::Action;
use rmk::types::keycode::HidKeyCode;

/// How long Caps Lock is held. QMK uses 80 ms for macOS for the same reason.
const CAPS_HOLD_MS: u64 = 80;
/// Time for the host to apply a layout switch before the next keystroke.
const LAYOUT_SETTLE_MS: u64 = 80;
/// Gap between the reports of a single keystroke.
const KEY_STEP_MS: u64 = 10;

/// Duration of one layout switch.
const SWITCH_MS: u64 = CAPS_HOLD_MS + LAYOUT_SETTLE_MS;
/// Extra wait in the macros, covering event delivery to this task.
const MARGIN_MS: u64 = 50;

/// Macro `Delay` for a layout toggle key.
pub const TOGGLE_DELAY_MS: u16 = (SWITCH_MS + MARGIN_MS) as u16;
/// Macro `Delay` for a wrapped symbol: switch, three report steps, switch back.
pub const SYMBOL_DELAY_MS: u16 = (2 * SWITCH_MS + 3 * KEY_STEP_MS + MARGIN_MS) as u16;

/// User action that only toggles the host layout (used by the layout keys).
pub const USER_TOGGLE: u8 = 1;

/// User actions for symbols missing in ЙЦУКЕН, typed through the EN layout.
/// The key and Shift state are the EN keystroke for the symbol.
pub const USER_AT: u8 = 2;
pub const USER_HASH: u8 = 3;
pub const USER_DOLLAR: u8 = 4;
pub const USER_CARET: u8 = 5;
pub const USER_AMPERSAND: u8 = 6;
pub const USER_LBRACKET: u8 = 7;
pub const USER_RBRACKET: u8 = 8;
pub const USER_LBRACE: u8 = 9;
pub const USER_RBRACE: u8 = 10;
pub const USER_QUOTE: u8 = 11;
pub const USER_GRAVE: u8 = 12;
pub const USER_PIPE: u8 = 13;

fn en_keystroke(user: u8) -> Option<(HidKeyCode, bool)> {
    Some(match user {
        USER_AT => (HidKeyCode::Kc2, true),
        USER_HASH => (HidKeyCode::Kc3, true),
        USER_DOLLAR => (HidKeyCode::Kc4, true),
        USER_CARET => (HidKeyCode::Kc6, true),
        USER_AMPERSAND => (HidKeyCode::Kc7, true),
        USER_LBRACKET => (HidKeyCode::LeftBracket, false),
        USER_RBRACKET => (HidKeyCode::RightBracket, false),
        USER_LBRACE => (HidKeyCode::LeftBracket, true),
        USER_RBRACE => (HidKeyCode::RightBracket, true),
        USER_QUOTE => (HidKeyCode::Quote, false),
        USER_GRAVE => (HidKeyCode::Grave, false),
        // Same code as EN shifted NonusBackslash.
        USER_PIPE => (HidKeyCode::NonusBackslash, true),
        _ => return None,
    })
}

const LSHIFT_BIT: u8 = 0x02;

async fn send(modifier: u8, key: Option<HidKeyCode>) {
    let mut keycodes = [0; 6];
    if let Some(key) = key {
        keycodes[0] = key as u8;
    }
    USB_REPORT_CHANNEL
        .send(Report::KeyboardReport(KeyboardReport {
            modifier,
            reserved: 0,
            leds: 0,
            keycodes,
        }))
        .await;
}

/// Toggle the host layout with a held Caps Lock press, then let it settle.
async fn switch_layout() {
    send(0, Some(HidKeyCode::CapsLock)).await;
    Timer::after_millis(CAPS_HOLD_MS).await;
    send(0, None).await;
    Timer::after_millis(LAYOUT_SETTLE_MS).await;
}

/// Type one EN keystroke: modifier first, then the key, then release.
async fn tap(key: HidKeyCode, shift: bool) {
    let modifier = if shift { LSHIFT_BIT } else { 0 };
    send(modifier, None).await;
    Timer::after_millis(KEY_STEP_MS).await;
    send(modifier, Some(key)).await;
    Timer::after_millis(KEY_STEP_MS).await;
    send(modifier, None).await;
    Timer::after_millis(KEY_STEP_MS).await;
    send(0, None).await;
}

pub async fn run() -> ! {
    let mut action_sub = ActionEvent::subscriber();
    loop {
        let event = action_sub.next_event().await;
        let Action::User(user) = event.action else {
            continue;
        };
        // RMK publishes ActionEvent on both press and release; act on press.
        if !event.keyboard_event.pressed {
            continue;
        }
        if user == USER_TOGGLE {
            switch_layout().await;
        } else if let Some((key, shift)) = en_keystroke(user) {
            switch_layout().await;
            tap(key, shift).await;
            switch_layout().await;
        }
    }
}
