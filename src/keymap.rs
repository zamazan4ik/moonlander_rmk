use rmk::config::Hand;
use rmk::types::action::{Action, KeyAction};
use rmk::types::keycode::HidKeyCode;
use rmk::types::modifier::ModifierCombination;
use rmk::{a, k, layer, light, macros, mo, mt, shifted, user, wm};

/// Index into behavior_config.morse.morses.
pub const _TD_ESC_EQL: u8 = 0;

pub(crate) const ROW: usize = 12;
pub(crate) const COL: usize = 7;
pub(crate) const NUM_LAYER: usize = 5;

/// Indices into the macro table in behavior.rs. The RU wrappers switch the OS
/// layout to EN, type a symbol missing in ЙЦУКЕН, and switch back.
pub(crate) const MACRO_AT: u8 = 2;
pub(crate) const MACRO_HASH: u8 = 5;
pub(crate) const MACRO_DOLLAR: u8 = 6;
pub(crate) const MACRO_CARET: u8 = 7;
pub(crate) const MACRO_AMPERSAND: u8 = 8;
pub(crate) const MACRO_LBRACKET: u8 = 9;
pub(crate) const MACRO_RBRACKET: u8 = 10;
pub(crate) const MACRO_LBRACE: u8 = 11;
pub(crate) const MACRO_RBRACE: u8 = 12;
pub(crate) const MACRO_QUOTE: u8 = 13;
pub(crate) const MACRO_GRAVE: u8 = 14;
pub(crate) const MACRO_PIPE: u8 = 15;

/// `macros!` only takes a literal; this takes a named index.
pub(crate) const fn macro_key(index: u8) -> KeyAction {
    KeyAction::Single(Action::TriggerMacro(index))
}

/// RU tap for the EN `/` (RAlt on hold) key: `/` is Shift+Backslash in ЙЦУКЕН.
const RU_SLASH_RALT: KeyAction = KeyAction::TapHold(
    Action::KeyWithModifier(HidKeyCode::Backslash, ModifierCombination::LSHIFT),
    Action::Modifier(ModifierCombination::RALT),
    u8::MAX,
);

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
        // English QWERTY. B/N are on the regular bottom alpha row.
        layer!([
            // Left half
            [k!(Escape), k!(Kc1), k!(Kc2), k!(Kc3), k!(Kc4), k!(Kc5), k!(Equal)],
            [shifted!(Minus), k!(Q), k!(W), k!(E), k!(R), k!(T), macros!(0)],
            [shifted!(Quote), k!(A), k!(S), k!(D), k!(F), k!(G), macros!(1)],
            [k!(Quote), k!(Z), k!(X), k!(C), k!(V), k!(B), a!(No)],
            [shifted!(NonusBackslash), a!(No), a!(No), a!(No), mo!(2), a!(No), a!(No)],
            // Three narrow thumbs, then the large red key.
            [k!(LShift), k!(Backspace), k!(Enter), k!(LCtrl), a!(No), a!(No), a!(No)],

            // Right half
            [k!(Equal), k!(Kc6), k!(Kc7), k!(Kc8), k!(Kc9), k!(Kc0), k!(Equal)],
            [k!(Delete), k!(Y), k!(U), k!(I), k!(O), k!(P), k!(Minus)],
            [k!(Enter), k!(H), k!(J), k!(K), k!(L), k!(Semicolon), shifted!(Semicolon)],
            [a!(No), k!(N), k!(M), k!(Comma), k!(Dot), mt!(Slash, ModifierCombination::RALT), k!(Semicolon)],
            [a!(No), a!(No), mo!(3), k!(Up), k!(Down), k!(RAlt), k!(Slash)],
            // Large red key, then three narrow thumbs (left to right).
            [a!(No), a!(No), a!(No), k!(LAlt), macros!(3), k!(Dot), k!(Space)]
        ]),
        // Russian ЙЦУКЕН (host: Windows/Linux "ru", macOS "Russian - PC").
        // Same physical symbols as EN: symbols that exist in ЙЦУКЕН are remapped,
        // missing ones go through EN wrapper macros. Х Ъ Ё Э are combos (behavior.rs).
        layer!([
            // Left half
            [k!(Escape), k!(Kc1), k!(Kc2), k!(Kc3), k!(Kc4), k!(Kc5), k!(Equal)],
            [shifted!(Minus), k!(Q), k!(W), k!(E), k!(R), k!(T), macros!(0)],
            [shifted!(Kc2), k!(A), k!(S), k!(D), k!(F), k!(G), macros!(1)],
            [macro_key(MACRO_QUOTE), k!(Z), k!(X), k!(C), k!(V), k!(B), a!(No)],
            [macro_key(MACRO_PIPE), a!(No), a!(No), a!(No), mo!(4), a!(No), a!(No)],
            // Three narrow thumbs, then the large red key.
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(No), a!(No), a!(No)],

            // Right half
            [k!(Equal), k!(Kc6), k!(Kc7), k!(Kc8), k!(Kc9), k!(Kc0), k!(Equal)],
            [k!(Delete), k!(Y), k!(U), k!(I), k!(O), k!(P), k!(Minus)],
            [k!(Enter), k!(H), k!(J), k!(K), k!(L), k!(Semicolon), shifted!(Kc6)],
            [a!(No), k!(N), k!(M), k!(Comma), k!(Dot), RU_SLASH_RALT, shifted!(Kc4)],
            [a!(No), a!(No), mo!(3), k!(Up), k!(Down), k!(RAlt), shifted!(Backslash)],
            // Large red key, then three narrow thumbs (left to right).
            [a!(No), a!(No), a!(No), a!(Transparent), macros!(4), k!(Slash), a!(Transparent)]
        ]),
        // SYM: shifted number symbols, brackets, function keys and a right-hand numpad.
        layer!([
            // Left half
            [k!(Escape), k!(F1), k!(F2), k!(F3), k!(F4), k!(F5), a!(Transparent)],
            [k!(Grave), shifted!(Kc1), shifted!(Kc2), shifted!(Kc3), shifted!(Kc4), shifted!(Kc5), a!(Transparent)],
            [a!(Transparent), shifted!(Kc6), shifted!(Kc7), shifted!(Kc8), shifted!(Kc9), shifted!(Kc0), a!(Transparent)],
            [a!(Transparent), a!(Transparent), k!(LeftBracket), k!(RightBracket), shifted!(LeftBracket), shifted!(RightBracket), a!(No)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(No), a!(No)],
            // Three narrow thumbs, then the large red key.
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(No), a!(No), a!(No)],

            // Right half
            [a!(Transparent), k!(F6), k!(F7), k!(F8), k!(F9), k!(F10), k!(F11)],
            [a!(Transparent), k!(Kc7), k!(Kc8), k!(Kc9), k!(Minus), k!(Slash), k!(F12)],
            [a!(Transparent), k!(Kc4), k!(Kc5), k!(Kc6), shifted!(Equal), shifted!(Kc8), k!(Backspace)],
            [a!(No), k!(Kc1), k!(Kc2), k!(Kc3), k!(Dot), k!(Equal), k!(Enter)],
            [a!(No), a!(No), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            // Large red key, then three narrow thumbs (left to right).
            [a!(No), a!(No), a!(No), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)]
        ]),
        // NAV: media, navigation, RGB controls. Outer top-right key enters DFU.
        layer!([
            // Left half
            [a!(Transparent), a!(Transparent), light!(RgbModeForward), a!(Transparent), light!(RgbVad), light!(RgbVai), a!(Transparent)],
            [light!(RgbHui), a!(Transparent), k!(AudioVolDown), k!(AudioVolUp), k!(AudioMute), a!(Transparent), light!(RgbHud)],
            [a!(Transparent), k!(MediaPrevTrack), k!(MediaNextTrack), k!(MediaStop), k!(MediaPlayPause), a!(Transparent), light!(RgbModeReverse)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(No)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(No), a!(No)],
            // Three narrow thumbs, then the large red key.
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(No), a!(No), a!(No)],

            // Right half
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), user!(0)],
            [a!(Transparent), k!(PageUp), k!(Home), k!(Up), k!(End), a!(Transparent), a!(Transparent)],
            [a!(Transparent), k!(PageDown), k!(Left), k!(Down), k!(Right), a!(Transparent), a!(Transparent)],
            [a!(No), a!(Transparent), wm!(Tab, ModifierCombination::new_from(false, false, false, true, true)), wm!(Tab, ModifierCombination::LCTRL), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(No), a!(No), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            // Large red key, then three narrow thumbs (left to right).
            [a!(No), a!(No), a!(No), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)]
        ]),
        // SYM-RU: SYM for the RU layer, with symbols remapped or wrapped for ЙЦУКЕН.
        layer!([
            // Left half
            [k!(Escape), k!(F1), k!(F2), k!(F3), k!(F4), k!(F5), a!(Transparent)],
            [macro_key(MACRO_GRAVE), shifted!(Kc1), macro_key(MACRO_AT), macro_key(MACRO_HASH), macro_key(MACRO_DOLLAR), shifted!(Kc5), a!(Transparent)],
            [a!(Transparent), macro_key(MACRO_CARET), macro_key(MACRO_AMPERSAND), shifted!(Kc8), shifted!(Kc9), shifted!(Kc0), a!(Transparent)],
            [a!(Transparent), a!(Transparent), macro_key(MACRO_LBRACKET), macro_key(MACRO_RBRACKET), macro_key(MACRO_LBRACE), macro_key(MACRO_RBRACE), a!(No)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(No), a!(No)],
            // Three narrow thumbs, then the large red key.
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(No), a!(No), a!(No)],

            // Right half
            [a!(Transparent), k!(F6), k!(F7), k!(F8), k!(F9), k!(F10), k!(F11)],
            [a!(Transparent), k!(Kc7), k!(Kc8), k!(Kc9), k!(Minus), shifted!(Backslash), k!(F12)],
            [a!(Transparent), k!(Kc4), k!(Kc5), k!(Kc6), shifted!(Equal), shifted!(Kc8), k!(Backspace)],
            [a!(No), k!(Kc1), k!(Kc2), k!(Kc3), k!(Slash), k!(Equal), k!(Enter)],
            [a!(No), a!(No), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            // Large red key, then three narrow thumbs (left to right).
            [a!(No), a!(No), a!(No), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)]
        ])
    ]
}
