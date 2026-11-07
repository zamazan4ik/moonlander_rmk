//! Host layout switching for the RU layer.
//!
//! The host toggles EN/RU on Caps Lock. RMK's macro `Tap` holds a key for only
//! 2 ms, and macOS ignores Caps Lock presses that short, so the switch was
//! randomly dropped or applied late. Here Caps Lock is held long enough and
//! given time to settle, with reports sent straight to the USB report channel.
//!
//! Each RU symbol key runs a tiny macro, `[TapAction(User(n)), Delay(..)]`
//! (`symbol_macro`). The `Delay` keeps RMK's key processing paused while this
//! task types, so keys pressed in the meantime are queued and come out in order,
//! in the right layout.
//!
//! The symbol key is held in EN for as long as the physical keys are held, so
//! the host's own key repeat works as on the EN layer. The hold ends on any
//! matrix change: a release, or the next key being pressed.

use embassy_time::{Duration, Instant, Timer};
use rmk::channel::USB_REPORT_CHANNEL;
use rmk::embassy_futures::select::{Either, select};
use rmk::event::{
    ActionEvent, EventSubscriber, KeyboardEvent, KeyboardEventPos, SubscribableEvent,
};
use rmk::hid::{KeyboardReport, Report};
use rmk::keyboard_macros::MacroOperation;
use rmk::types::action::Action;
use rmk::types::keycode::HidKeyCode;

use crate::keymap::{COL, ROW};

/// How long Caps Lock is held. QMK uses 80 ms for macOS for the same reason.
const CAPS_HOLD_MS: u64 = 80;
/// Time for the host to apply a layout switch before the next keystroke.
const LAYOUT_SETTLE_MS: u64 = 80;
/// Gap between the reports of a single keystroke.
const KEY_STEP_MS: u64 = 10;
/// How late after a symbol the next key can be pressed and still be held back
/// until the host is in RU again. Covers rolling from a symbol onto a letter.
const ROLLOVER_MS: u64 = 200;

/// Duration of one layout switch.
const SWITCH_MS: u64 = CAPS_HOLD_MS + LAYOUT_SETTLE_MS;
/// Extra wait in the macros, covering event delivery to this task.
const MARGIN_MS: u64 = 50;

/// Macro `Delay` for a layout toggle key.
const TOGGLE_DELAY_MS: u16 = (SWITCH_MS + MARGIN_MS) as u16;
/// Macro `Delay` for a wrapped symbol: switch, press, rollover window,
/// release, switch back.
const SYMBOL_DELAY_MS: u16 = (2 * SWITCH_MS + 3 * KEY_STEP_MS + ROLLOVER_MS + MARGIN_MS) as u16;

/// User action that only toggles the host layout. User 0 is the DFU key.
const USER_TOGGLE: u8 = 1;
/// User action of the first `EnSymbol`; the rest follow in order.
const USER_FIRST_SYMBOL: u8 = 2;

/// Symbols missing in ЙЦУКЕН, typed through the EN layout.
///
/// The variant order is the order of the User actions and of the symbol
/// macros (see `behavior::Macro`). Append new symbols at the end and bump
/// `LAYOUT_TAG`, so stored macros match the new defaults.
#[derive(Clone, Copy)]
pub enum EnSymbol {
    At,
    Hash,
    Dollar,
    Caret,
    Ampersand,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Quote,
    Grave,
    /// Same code as EN shifted NonusBackslash.
    Pipe,
}

impl EnSymbol {
    /// Every symbol, in variant order.
    pub const ALL: [Self; 12] = [
        Self::At,
        Self::Hash,
        Self::Dollar,
        Self::Caret,
        Self::Ampersand,
        Self::LeftBracket,
        Self::RightBracket,
        Self::LeftBrace,
        Self::RightBrace,
        Self::Quote,
        Self::Grave,
        Self::Pipe,
    ];

    /// The EN keystroke: key and Shift.
    const fn keystroke(self) -> (HidKeyCode, bool) {
        match self {
            Self::At => (HidKeyCode::Kc2, true),
            Self::Hash => (HidKeyCode::Kc3, true),
            Self::Dollar => (HidKeyCode::Kc4, true),
            Self::Caret => (HidKeyCode::Kc6, true),
            Self::Ampersand => (HidKeyCode::Kc7, true),
            Self::LeftBracket => (HidKeyCode::LeftBracket, false),
            Self::RightBracket => (HidKeyCode::RightBracket, false),
            Self::LeftBrace => (HidKeyCode::LeftBracket, true),
            Self::RightBrace => (HidKeyCode::RightBracket, true),
            Self::Quote => (HidKeyCode::Quote, false),
            Self::Grave => (HidKeyCode::Grave, false),
            Self::Pipe => (HidKeyCode::NonusBackslash, true),
        }
    }

    const fn user(self) -> u8 {
        USER_FIRST_SYMBOL + self as u8
    }

    fn from_user(user: u8) -> Option<Self> {
        Self::ALL
            .get(user.checked_sub(USER_FIRST_SYMBOL)? as usize)
            .copied()
    }
}

// `from_user` indexes ALL by variant number.
const _: () = {
    let mut i = 0;
    while i < EnSymbol::ALL.len() {
        assert!(
            EnSymbol::ALL[i] as usize == i,
            "EnSymbol::ALL is out of variant order"
        );
        i += 1;
    }
};

/// Macro typing `symbol` on the RU layer. The `Delay` keeps RMK's key
/// processing paused while this task types, so later keys stay in order.
pub const fn symbol_macro(symbol: EnSymbol) -> [MacroOperation; 2] {
    [
        MacroOperation::TapAction(Action::User(symbol.user())),
        MacroOperation::Delay(SYMBOL_DELAY_MS),
    ]
}

/// Macro toggling the host layout and switching the keymap to `layer`.
pub const fn toggle_macro(layer: u8) -> [MacroOperation; 3] {
    [
        MacroOperation::TapAction(Action::User(USER_TOGGLE)),
        MacroOperation::Delay(TOGGLE_DELAY_MS),
        MacroOperation::TapAction(Action::LayerToggleOnly(layer)),
    ]
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

struct Switcher {
    actions: <ActionEvent as SubscribableEvent>::Subscriber,
    keys: <KeyboardEvent as SubscribableEvent>::Subscriber,
    /// Physical key state, one bit per column.
    pressed: [u8; ROW],
    /// Set on any matrix change; cleared when a symbol starts.
    changed: bool,
}

impl Switcher {
    fn on_key(&mut self, event: KeyboardEvent) {
        if let KeyboardEventPos::Key(pos) = event.pos
            && (pos.row as usize) < ROW
            && (pos.col as usize) < COL
        {
            let bit = 1 << pos.col;
            if event.pressed {
                self.pressed[pos.row as usize] |= bit;
            } else {
                self.pressed[pos.row as usize] &= !bit;
            }
        }
        self.changed = true;
    }

    fn is_pressed(&self, event: &KeyboardEvent) -> bool {
        match event.pos {
            KeyboardEventPos::Key(pos) if (pos.row as usize) < ROW => {
                self.pressed[pos.row as usize] & (1 << pos.col) != 0
            }
            _ => false,
        }
    }

    /// Wait `ms` while keeping the key state current.
    async fn wait_ms(&mut self, ms: u64) {
        let deadline = Instant::now() + Duration::from_millis(ms);
        loop {
            match select(Timer::at(deadline), self.keys.next_event()).await {
                Either::First(()) => return,
                Either::Second(event) => self.on_key(event),
            }
        }
    }

    /// Wait until any key is pressed or released.
    async fn wait_change(&mut self) {
        while !self.changed {
            let event = self.keys.next_event().await;
            self.on_key(event);
        }
    }

    /// Toggle the host layout with a held Caps Lock press, then let it settle.
    async fn switch_layout(&mut self) {
        send(0, Some(HidKeyCode::CapsLock)).await;
        self.wait_ms(CAPS_HOLD_MS).await;
        send(0, None).await;
        self.wait_ms(LAYOUT_SETTLE_MS).await;
    }

    /// Type an EN symbol, held for as long as the physical keys are held.
    async fn type_symbol(&mut self, trigger: &KeyboardEvent, symbol: EnSymbol) {
        // A trigger already released by now was a tap.
        self.changed = !self.is_pressed(trigger);
        self.switch_layout().await;

        let (key, shift) = symbol.keystroke();
        let modifier = if shift { LSHIFT_BIT } else { 0 };
        send(modifier, None).await;
        self.wait_ms(KEY_STEP_MS).await;
        send(modifier, Some(key)).await;
        self.wait_ms(KEY_STEP_MS).await;
        self.wait_change().await;
        send(modifier, None).await;
        self.wait_ms(KEY_STEP_MS).await;
        send(0, None).await;

        self.switch_layout().await;
    }

    async fn run(&mut self) -> ! {
        loop {
            // Key events first, so the key state is current when an action
            // arrives: RMK publishes the key event before processing it.
            let event = match select(self.keys.next_event(), self.actions.next_event()).await {
                Either::First(event) => {
                    self.on_key(event);
                    continue;
                }
                Either::Second(event) => event,
            };
            let Action::User(user) = event.action else {
                continue;
            };
            // RMK publishes ActionEvent on both press and release; act on press.
            if !event.keyboard_event.pressed {
                continue;
            }
            if user == USER_TOGGLE {
                self.switch_layout().await;
            } else if let Some(symbol) = EnSymbol::from_user(user) {
                self.type_symbol(&event.keyboard_event, symbol).await;
            }
        }
    }
}

pub async fn run() -> ! {
    Switcher {
        actions: ActionEvent::subscriber(),
        keys: KeyboardEvent::subscriber(),
        pressed: [0; ROW],
        changed: false,
    }
    .run()
    .await
}
