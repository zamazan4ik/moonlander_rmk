use rmk::config::{BehaviorConfig, KeyboardMacrosConfig};
use rmk::heapless::Vec;
use rmk::keyboard::combo::{Combo, ComboConfig};
use rmk::keyboard_macros::{MacroOperation, define_macro_sequences};
use rmk::types::action::Action;
use rmk::types::constants::MACRO_SPACE_SIZE;
use rmk::types::keycode::HidKeyCode;
use rmk::{k, shifted};

use crate::keymap::{MACRO_AMPERSAND, MACRO_AT, MACRO_CARET, MACRO_DOLLAR, MACRO_HASH, macro_key};

pub fn get_default_behavior() -> BehaviorConfig {
    let mut behavior_config = BehaviorConfig::default();

    // TD_ESC_EQL: tap = Escape, double-tap = Equal.
    /*let _ = behavior_config.morse.morses.push(Morse::new_from_vial(
        Action::Key(KeyCode::Hid(HidKeyCode::Escape)),
        Action::No,
        Action::No,
        Action::Key(KeyCode::Hid(HidKeyCode::Equal)),
        MorseProfile::const_default(),
    ));*/

    // Combo group for special symbols: number + top row letter -> shifted number.
    // ! % * ( ) are the same in both layouts, so they fire on every layer.
    // @ # $ ^ & are EN-only here; the RU layer has its own copies below.
    behavior_config.combo.combos[0] = Some(Combo::new(ComboConfig::new(
        [k!(Kc1), k!(Q)],
        shifted!(Kc1),
        None,
    )));
    behavior_config.combo.combos[1] = Some(Combo::new(ComboConfig::new(
        [k!(Kc2), k!(W)],
        shifted!(Kc2),
        Some(0),
    )));
    behavior_config.combo.combos[2] = Some(Combo::new(ComboConfig::new(
        [k!(Kc3), k!(E)],
        shifted!(Kc3),
        Some(0),
    )));
    behavior_config.combo.combos[3] = Some(Combo::new(ComboConfig::new(
        [k!(Kc4), k!(R)],
        shifted!(Kc4),
        Some(0),
    )));
    behavior_config.combo.combos[4] = Some(Combo::new(ComboConfig::new(
        [k!(Kc5), k!(T)],
        shifted!(Kc5),
        None,
    )));
    behavior_config.combo.combos[5] = Some(Combo::new(ComboConfig::new(
        [k!(Kc6), k!(Y)],
        shifted!(Kc6),
        Some(0),
    )));
    behavior_config.combo.combos[6] = Some(Combo::new(ComboConfig::new(
        [k!(Kc7), k!(U)],
        shifted!(Kc7),
        Some(0),
    )));
    behavior_config.combo.combos[7] = Some(Combo::new(ComboConfig::new(
        [k!(Kc8), k!(I)],
        shifted!(Kc8),
        None,
    )));
    behavior_config.combo.combos[8] = Some(Combo::new(ComboConfig::new(
        [k!(Kc9), k!(O)],
        shifted!(Kc9),
        None,
    )));
    behavior_config.combo.combos[9] = Some(Combo::new(ComboConfig::new(
        [k!(Kc0), k!(P)],
        shifted!(Kc0),
        None,
    )));

    // Combo group for F-row: two near buttons at the number row -> F-button
    behavior_config.combo.combos[10] = Some(Combo::new(ComboConfig::new(
        [k!(Escape), k!(Kc1)],
        k!(F1),
        None,
    )));
    behavior_config.combo.combos[11] = Some(Combo::new(ComboConfig::new(
        [k!(Kc1), k!(Kc2)],
        k!(F2),
        None,
    )));
    behavior_config.combo.combos[12] = Some(Combo::new(ComboConfig::new(
        [k!(Kc2), k!(Kc3)],
        k!(F3),
        None,
    )));
    behavior_config.combo.combos[13] = Some(Combo::new(ComboConfig::new(
        [k!(Kc3), k!(Kc4)],
        k!(F4),
        None,
    )));
    behavior_config.combo.combos[14] = Some(Combo::new(ComboConfig::new(
        [k!(Kc4), k!(Kc5)],
        k!(F5),
        None,
    )));
    behavior_config.combo.combos[15] = Some(Combo::new(ComboConfig::new(
        [k!(Kc5), k!(Equal)],
        k!(F6),
        None,
    )));
    behavior_config.combo.combos[16] = Some(Combo::new(ComboConfig::new(
        [k!(Equal), k!(Kc6)],
        k!(F7),
        None,
    )));
    behavior_config.combo.combos[17] = Some(Combo::new(ComboConfig::new(
        [k!(Kc6), k!(Kc7)],
        k!(F8),
        None,
    )));
    behavior_config.combo.combos[18] = Some(Combo::new(ComboConfig::new(
        [k!(Kc7), k!(Kc8)],
        k!(F9),
        None,
    )));
    behavior_config.combo.combos[19] = Some(Combo::new(ComboConfig::new(
        [k!(Kc8), k!(Kc9)],
        k!(F10),
        None,
    )));
    behavior_config.combo.combos[20] = Some(Combo::new(ComboConfig::new(
        [k!(Kc9), k!(Kc0)],
        k!(F11),
        None,
    )));
    behavior_config.combo.combos[21] = Some(Combo::new(ComboConfig::new(
        [k!(Kc0), k!(Equal)],
        k!(F12),
        None,
    )));

    // RU copies of the symbol combos: these symbols are missing in ЙЦУКЕН,
    // so they go through the EN wrapper macros.
    behavior_config.combo.combos[22] = Some(Combo::new(ComboConfig::new(
        [k!(Kc2), k!(W)],
        macro_key(MACRO_AT),
        Some(1),
    )));
    behavior_config.combo.combos[23] = Some(Combo::new(ComboConfig::new(
        [k!(Kc3), k!(E)],
        macro_key(MACRO_HASH),
        Some(1),
    )));
    behavior_config.combo.combos[24] = Some(Combo::new(ComboConfig::new(
        [k!(Kc4), k!(R)],
        macro_key(MACRO_DOLLAR),
        Some(1),
    )));
    behavior_config.combo.combos[25] = Some(Combo::new(ComboConfig::new(
        [k!(Kc6), k!(Y)],
        macro_key(MACRO_CARET),
        Some(1),
    )));
    behavior_config.combo.combos[26] = Some(Combo::new(ComboConfig::new(
        [k!(Kc7), k!(U)],
        macro_key(MACRO_AMPERSAND),
        Some(1),
    )));

    // Russian letters without a dedicated key: vertical chords, as in the QMK firmware.
    // Х = Н+Р (Y+H), Ъ = О+Ь (J+M), Ё = А+К (F+R), Э = Р+Т (H+N). Hold Shift for capitals.
    behavior_config.combo.combos[27] = Some(Combo::new(ComboConfig::new(
        [k!(Y), k!(H)],
        k!(LeftBracket),
        Some(1),
    )));
    behavior_config.combo.combos[28] = Some(Combo::new(ComboConfig::new(
        [k!(J), k!(M)],
        k!(RightBracket),
        Some(1),
    )));
    behavior_config.combo.combos[29] = Some(Combo::new(ComboConfig::new(
        [k!(F), k!(R)],
        k!(Grave),
        Some(1),
    )));
    behavior_config.combo.combos[30] = Some(Combo::new(ComboConfig::new(
        [k!(H), k!(N)],
        k!(Quote),
        Some(1),
    )));

    behavior_config.keyboard_macros = KeyboardMacrosConfig::new(get_macro_sequences());

    behavior_config
}

/// Wrap an EN keystroke for the RU layer: switch the OS layout to EN, type `ops`,
/// switch back. The delays let the OS apply the layout change before and after the
/// keystroke; raise them if a host drops or mistypes symbols.
fn wrap_en(ops: &[MacroOperation]) -> Vec<MacroOperation, 8> {
    let mut seq = Vec::new();
    let _ = seq.push(MacroOperation::Tap(HidKeyCode::CapsLock));
    let _ = seq.push(MacroOperation::Delay(40));
    let _ = seq.extend_from_slice(ops);
    let _ = seq.push(MacroOperation::Delay(20));
    let _ = seq.push(MacroOperation::Tap(HidKeyCode::CapsLock));
    seq
}

/// A one-character `Text` op: a single byte in the macro space, and RMK adds Shift itself.
const fn text(key: HidKeyCode, shift: bool) -> MacroOperation {
    MacroOperation::Text(key, shift)
}

/// Serialize one macro and append it, including its End byte, to `out`.
///
/// `define_macro_sequences` only takes `Vec<_, MACRO_SPACE_SIZE>`, which is ~2 KB
/// each. Passing all macros at once puts every one of them on the stack and
/// overflows the 32 KB RAM at boot, so they are serialized one at a time here.
/// Encoded macros never contain a 0 byte except End.
#[inline(never)]
fn append_macro(out: &mut [u8; MACRO_SPACE_SIZE], len: &mut usize, ops: &[MacroOperation]) {
    let seq: Vec<MacroOperation, MACRO_SPACE_SIZE> = Vec::from_slice(ops).expect("too many elements");
    let bin = define_macro_sequences(core::slice::from_ref(&seq));
    let n = bin.iter().position(|&b| b == 0).expect("macro has no End byte") + 1;
    out.get_mut(*len..*len + n)
        .expect("macros exceed macro_space_size")
        .copy_from_slice(&bin[..n]);
    *len += n;
}

/// Macro order must match the MACRO_* indices in keymap.rs.
pub fn get_macro_sequences() -> [u8; MACRO_SPACE_SIZE] {
    let mut out = [0; MACRO_SPACE_SIZE];
    let mut len = 0;

    // Copy & Paste macros
    // TODO: make them OS-agnostic - support Copy&Paste for macOS/Windows/Linux (including ctrl+shift+c/v)
    append_macro(&mut out, &mut len, &[
        MacroOperation::Press(HidKeyCode::LGui),
        MacroOperation::Tap(HidKeyCode::C),
        MacroOperation::Release(HidKeyCode::LGui),
    ]);
    append_macro(&mut out, &mut len, &[
        MacroOperation::Press(HidKeyCode::LGui),
        MacroOperation::Tap(HidKeyCode::V),
        MacroOperation::Release(HidKeyCode::LGui),
    ]);
    append_macro(&mut out, &mut len, &wrap_en(&[text(HidKeyCode::Kc2, true)])); // @
    append_macro(&mut out, &mut len, &[
        MacroOperation::Tap(HidKeyCode::CapsLock),
        MacroOperation::TapAction(Action::LayerToggleOnly(1)),
    ]);
    append_macro(&mut out, &mut len, &[
        MacroOperation::Tap(HidKeyCode::CapsLock),
        MacroOperation::TapAction(Action::LayerToggleOnly(0)),
    ]);

    // Symbols missing in ЙЦУКЕН, typed through the EN layout.
    for op in [
        text(HidKeyCode::Kc3, true),           // #
        text(HidKeyCode::Kc4, true),           // $
        text(HidKeyCode::Kc6, true),           // ^
        text(HidKeyCode::Kc7, true),           // &
        text(HidKeyCode::LeftBracket, false),  // [
        text(HidKeyCode::RightBracket, false), // ]
        text(HidKeyCode::LeftBracket, true),   // {
        text(HidKeyCode::RightBracket, true),  // }
        text(HidKeyCode::Quote, false),        // '
        text(HidKeyCode::Grave, false),        // `
    ] {
        append_macro(&mut out, &mut len, &wrap_en(&[op]));
    }
    // Same code as EN shifted NonusBackslash, which has no ASCII Text form.
    append_macro(&mut out, &mut len, &wrap_en(&[
        MacroOperation::Press(HidKeyCode::LShift),
        MacroOperation::Tap(HidKeyCode::NonusBackslash),
        MacroOperation::Release(HidKeyCode::LShift),
    ]));

    out
}
