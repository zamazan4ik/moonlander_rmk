use rmk::config::Hand;
use rmk::types::action::KeyAction;
use rmk::types::modifier::ModifierCombination;
use rmk::{a, k, kbctrl, layer, light, lt, mt, shifted, user, wm};

/// Index into behavior_config.morse.morses.
pub const TD_ESC_EQL: u8 = 0;

pub(crate) const ROW: usize = 12;
pub(crate) const COL: usize = 7;
pub(crate) const NUM_LAYER: usize = 3;

/// Rows 0-4 / 6-10 belong to the left / right hand.
/// Thumb rows stay Unknown so same-hand alphas do not force thumb mod-taps to tap.
#[rustfmt::skip]
pub const HAND_MAP: [[Hand; COL]; ROW] = [
    [Hand::Left; COL], [Hand::Left; COL], [Hand::Left; COL],
    [Hand::Left; COL], [Hand::Left; COL], [Hand::Unknown; COL],
    [Hand::Right; COL], [Hand::Right; COL], [Hand::Right; COL],
    [Hand::Right; COL], [Hand::Right; COL], [Hand::Unknown; COL],
];

/// Moonlander Mark I: 36 switches per half in a 12x7 logical matrix.
/// Left: rows 0-2 cols 0-6, row 3 cols 0-5, row 4 cols 0-4, thumbs (5,0..3).
/// Right: rows 6-8 cols 0-6, row 9 cols 1-6, row 10 cols 2-6, thumbs (11,3..6).
/// No disables an unused cell; Transparent inherits the lower layer.
/// The extra right-hand column is the INNER column (0), so 6/Y/H start at
/// column 1, just like 1/Q/A on the left. Row 9 already starts at column 1.
#[rustfmt::skip]
pub const fn get_default_keymap() -> [[[KeyAction; COL]; ROW]; NUM_LAYER] {
    [
        // BASE: QWERTY. B/N are on the regular bottom alpha row.
        layer!([
            // Left half
            [KeyAction::Morse(TD_ESC_EQL), k!(Kc1), k!(Kc2), k!(Kc3), k!(Kc4), k!(Kc5), k!(Equal)],
            [kbctrl!(CapsWordToggle), k!(Q), k!(W), k!(E), k!(R), k!(T), k!(LeftBracket)],
            [mt!(Backspace, ModifierCombination::LSHIFT), k!(A), k!(S), k!(D), k!(F), k!(G), k!(RightBracket)],
            [k!(LGui), mt!(Z, ModifierCombination::LALT), k!(X), k!(C), k!(V), k!(B), a!(No)],
            [k!(LCtrl), k!(LAlt), k!(Left), k!(Right), k!(Home), a!(No), a!(No)],
            // Three narrow thumbs, then the large red key.
            [lt!(1, Enter), mt!(Tab, ModifierCombination::LCTRL), k!(Delete), k!(LGui), a!(No), a!(No), a!(No)],
            // Right half
            [k!(Equal), k!(Kc6), k!(Kc7), k!(Kc8), k!(Kc9), k!(Kc0), k!(Minus)],
            [k!(Delete), k!(Y), k!(U), k!(I), k!(O), k!(P), k!(Backslash)],
            [k!(Enter), k!(H), k!(J), k!(K), k!(L), k!(Semicolon), mt!(Quote, ModifierCombination::RSHIFT)],
            [a!(No), k!(N), k!(M), k!(Comma), k!(Dot), mt!(Slash, ModifierCombination::RALT), k!(RCtrl)],
            [a!(No), a!(No), k!(End), k!(Up), k!(Down), k!(RAlt), k!(RGui)],
            // Large red key, then three narrow thumbs (left to right).
            [a!(No), a!(No), a!(No), k!(Escape), k!(Tab), mt!(Backspace, ModifierCombination::RSHIFT), lt!(2, Space)]
        ]),
        // SYM: shifted number symbols, brackets, function keys and a right-hand numpad.
        layer!([
            [k!(Escape), k!(F1), k!(F2), k!(F3), k!(F4), k!(F5), a!(Transparent)],
            [k!(Grave), shifted!(Kc1), shifted!(Kc2), shifted!(Kc3), shifted!(Kc4), shifted!(Kc5), a!(Transparent)],
            [a!(Transparent), shifted!(Kc6), shifted!(Kc7), shifted!(Kc8), shifted!(Kc9), shifted!(Kc0), a!(Transparent)],
            [a!(Transparent), a!(Transparent), k!(LeftBracket), k!(RightBracket), shifted!(LeftBracket), shifted!(RightBracket), a!(No)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(No), a!(No)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(No), a!(No), a!(No)],
            [a!(Transparent), k!(F6), k!(F7), k!(F8), k!(F9), k!(F10), k!(F11)],
            [a!(Transparent), k!(Kc7), k!(Kc8), k!(Kc9), k!(Minus), k!(Slash), k!(F12)],
            [a!(Transparent), k!(Kc4), k!(Kc5), k!(Kc6), shifted!(Equal), shifted!(Kc8), k!(Backspace)],
            [a!(No), k!(Kc1), k!(Kc2), k!(Kc3), k!(Dot), k!(Equal), k!(Enter)],
            [a!(No), a!(No), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(No), a!(No), a!(No), a!(Transparent), a!(Transparent), a!(Transparent), k!(Kc0)]
        ]),
        // NAV: media, navigation, RGB controls. Outer top-right key enters DFU.
        layer!([
            [a!(Transparent), a!(Transparent), light!(RgbModeForward), a!(Transparent), light!(RgbVad), light!(RgbVai), a!(Transparent)],
            [light!(RgbHui), a!(Transparent), k!(AudioVolDown), k!(AudioVolUp), k!(AudioMute), a!(Transparent), light!(RgbHud)],
            [a!(Transparent), k!(MediaPrevTrack), k!(MediaNextTrack), k!(MediaStop), k!(MediaPlayPause), a!(Transparent), light!(RgbModeReverse)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(No)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(No), a!(No)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(No), a!(No), a!(No)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), user!(0)],
            [a!(Transparent), k!(PageUp), k!(Home), k!(Up), k!(End), a!(Transparent), a!(Transparent)],
            [a!(Transparent), k!(PageDown), k!(Left), k!(Down), k!(Right), a!(Transparent), a!(Transparent)],
            [a!(No), a!(Transparent), wm!(Tab, ModifierCombination::new_from(false, false, false, true, true)), wm!(Tab, ModifierCombination::LCTRL), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(No), a!(No), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(No), a!(No), a!(No), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)]
        ])
    ]
}
