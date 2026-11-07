use rmk::config::{BehaviorConfig, KeyboardMacrosConfig};
use rmk::heapless::Vec;
use rmk::keyboard::combo::{Combo, ComboConfig};
use rmk::keyboard_macros::{MacroOperation, define_macro_sequences};
use rmk::types::action::{Action, KeyAction};
use rmk::types::constants::MACRO_SPACE_SIZE;
use rmk::types::keycode::{HidKeyCode, KeyCode};
use rmk::types::modifier::ModifierCombination;

use crate::layout_switch::{self, EnSymbol};

/// Default macros. The index of each one is its position in the macro table:
/// the fixed macros first, then one per `EnSymbol`, in variant order.
#[derive(Clone, Copy)]
pub(crate) enum Macro {
    /// Cmd+C.
    Copy,
    /// Cmd+V.
    Paste,
    /// Switch the host and the keymap to RU.
    LayoutRu,
    /// Switch the host and the keymap to EN.
    LayoutEn,
    /// Type a symbol missing in ЙЦУКЕН through the EN layout.
    Symbol(EnSymbol),
}

impl Macro {
    const FIXED: [Self; 4] = [Self::Copy, Self::Paste, Self::LayoutRu, Self::LayoutEn];

    const fn index(self) -> u8 {
        match self {
            Self::Copy => 0,
            Self::Paste => 1,
            Self::LayoutRu => 2,
            Self::LayoutEn => 3,
            Self::Symbol(symbol) => Self::FIXED.len() as u8 + symbol as u8,
        }
    }

    /// Keymap action that runs this macro.
    pub(crate) const fn key(self) -> KeyAction {
        KeyAction::Single(Action::TriggerMacro(self.index()))
    }

    fn ops(self) -> Vec<MacroOperation, 3> {
        let ops: &[MacroOperation] = match self {
            // TODO: make them OS-agnostic - support Copy&Paste for macOS/Windows/Linux (including ctrl+shift+c/v)
            Self::Copy => &[
                MacroOperation::Press(HidKeyCode::LGui),
                MacroOperation::Tap(HidKeyCode::C),
                MacroOperation::Release(HidKeyCode::LGui),
            ],
            Self::Paste => &[
                MacroOperation::Press(HidKeyCode::LGui),
                MacroOperation::Tap(HidKeyCode::V),
                MacroOperation::Release(HidKeyCode::LGui),
            ],
            Self::LayoutRu => &const { layout_switch::toggle_macro(RU_LAYER) },
            Self::LayoutEn => &const { layout_switch::toggle_macro(EN_LAYER) },
            Self::Symbol(symbol) => &layout_switch::symbol_macro(symbol),
        };
        Vec::from_slice(ops).expect("too many elements")
    }
}

// The macro table is FIXED then EnSymbol::ALL, so FIXED must be in index order.
const _: () = {
    let mut i = 0;
    while i < Macro::FIXED.len() {
        assert!(
            Macro::FIXED[i].index() as usize == i,
            "Macro::FIXED is out of index order"
        );
        i += 1;
    }
};

/// Keymap action for a symbol typed through the EN layout (RU layers only).
pub(crate) const fn via_en(symbol: EnSymbol) -> KeyAction {
    Macro::Symbol(symbol).key()
}

const fn key(code: HidKeyCode) -> KeyAction {
    KeyAction::Single(Action::Key(KeyCode::Hid(code)))
}

const fn shifted(code: HidKeyCode) -> KeyAction {
    KeyAction::Single(Action::KeyWithModifier(code, ModifierCombination::LSHIFT))
}

const EN_LAYER: u8 = 0;
const RU_LAYER: u8 = 1;

/// Number + top-row letter -> the shifted number. Symbols missing in ЙЦУКЕН
/// get an EN-only combo plus an RU one typed through the EN layout; the rest
/// are the same in both layouts and fire on every layer.
const NUMBER_SYMBOL_COMBOS: [(HidKeyCode, HidKeyCode, Option<EnSymbol>); 10] = [
    (HidKeyCode::Kc1, HidKeyCode::Q, None),
    (HidKeyCode::Kc2, HidKeyCode::W, Some(EnSymbol::At)),
    (HidKeyCode::Kc3, HidKeyCode::E, Some(EnSymbol::Hash)),
    (HidKeyCode::Kc4, HidKeyCode::R, Some(EnSymbol::Dollar)),
    (HidKeyCode::Kc5, HidKeyCode::T, None),
    (HidKeyCode::Kc6, HidKeyCode::Y, Some(EnSymbol::Caret)),
    (HidKeyCode::Kc7, HidKeyCode::U, Some(EnSymbol::Ampersand)),
    (HidKeyCode::Kc8, HidKeyCode::I, None),
    (HidKeyCode::Kc9, HidKeyCode::O, None),
    (HidKeyCode::Kc0, HidKeyCode::P, None),
];

/// The number row, left to right. Each pair of neighbours -> F1..F12.
const NUMBER_ROW: [HidKeyCode; 13] = [
    HidKeyCode::Escape,
    HidKeyCode::Kc1,
    HidKeyCode::Kc2,
    HidKeyCode::Kc3,
    HidKeyCode::Kc4,
    HidKeyCode::Kc5,
    HidKeyCode::Equal,
    HidKeyCode::Kc6,
    HidKeyCode::Kc7,
    HidKeyCode::Kc8,
    HidKeyCode::Kc9,
    HidKeyCode::Kc0,
    HidKeyCode::Equal,
];
const F_KEYS: [HidKeyCode; 12] = [
    HidKeyCode::F1,
    HidKeyCode::F2,
    HidKeyCode::F3,
    HidKeyCode::F4,
    HidKeyCode::F5,
    HidKeyCode::F6,
    HidKeyCode::F7,
    HidKeyCode::F8,
    HidKeyCode::F9,
    HidKeyCode::F10,
    HidKeyCode::F11,
    HidKeyCode::F12,
];

/// Russian letters without a dedicated key: vertical chords, as in the QMK firmware.
/// Х = Н+Р (Y+H), Ъ = О+Ь (J+M), Ё = А+К (F+R), Э = Р+Т (H+N). Hold Shift for capitals.
const RU_LETTER_COMBOS: [(HidKeyCode, HidKeyCode, HidKeyCode); 4] = [
    (HidKeyCode::Y, HidKeyCode::H, HidKeyCode::LeftBracket),
    (HidKeyCode::J, HidKeyCode::M, HidKeyCode::RightBracket),
    (HidKeyCode::F, HidKeyCode::R, HidKeyCode::Grave),
    (HidKeyCode::H, HidKeyCode::N, HidKeyCode::Quote),
];

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

    let mut slots = behavior_config.combo.combos.iter_mut();
    let mut add = |a: HidKeyCode, b: HidKeyCode, output: KeyAction, layer: Option<u8>| {
        *slots.next().expect("raise combo_max_num in keyboard.toml") = Some(Combo::new(
            ComboConfig::new([key(a), key(b)], output, layer),
        ));
    };

    for (number, letter, ru) in NUMBER_SYMBOL_COMBOS {
        match ru {
            None => add(number, letter, shifted(number), None),
            Some(symbol) => {
                add(number, letter, shifted(number), Some(EN_LAYER));
                add(number, letter, via_en(symbol), Some(RU_LAYER));
            }
        }
    }
    for (pair, f_key) in NUMBER_ROW.windows(2).zip(F_KEYS) {
        add(pair[0], pair[1], key(f_key), None);
    }
    for (a, b, letter) in RU_LETTER_COMBOS {
        add(a, b, key(letter), Some(RU_LAYER));
    }

    behavior_config.keyboard_macros = KeyboardMacrosConfig::new(get_macro_sequences());

    behavior_config
}

/// Serialize one macro and append it, including its End byte, to `out`.
///
/// `define_macro_sequences` only takes `Vec<_, MACRO_SPACE_SIZE>`, which is ~2 KB
/// each. Passing all macros at once puts every one of them on the stack and
/// overflows the 32 KB RAM at boot, so they are serialized one at a time here.
/// Encoded macros never contain a 0 byte except End.
#[inline(never)]
fn append_macro(out: &mut [u8; MACRO_SPACE_SIZE], len: &mut usize, ops: &[MacroOperation]) {
    let seq: Vec<MacroOperation, MACRO_SPACE_SIZE> =
        Vec::from_slice(ops).expect("too many elements");
    let bin = define_macro_sequences(core::slice::from_ref(&seq));
    let n = bin
        .iter()
        .position(|&b| b == 0)
        .expect("macro has no End byte")
        + 1;
    out.get_mut(*len..*len + n)
        .expect("macros exceed macro_space_size")
        .copy_from_slice(&bin[..n]);
    *len += n;
}

pub fn get_macro_sequences() -> [u8; MACRO_SPACE_SIZE] {
    let mut out = [0; MACRO_SPACE_SIZE];
    let mut len = 0;
    for m in Macro::FIXED
        .into_iter()
        .chain(EnSymbol::ALL.map(Macro::Symbol))
    {
        append_macro(&mut out, &mut len, &m.ops());
    }
    out
}
