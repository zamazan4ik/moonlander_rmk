use rmk::config::{BehaviorConfig, KeyboardMacrosConfig};
use rmk::heapless::Vec;
use rmk::keyboard::combo::{Combo, ComboConfig};
use rmk::keyboard_macros::{MacroOperation, define_macro_sequences};
use rmk::types::constants::MACRO_SPACE_SIZE;
use rmk::types::keycode::HidKeyCode;
use rmk::{k, shifted};

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

    // Combo group for special symbols: number + top row letter -> shifted number
    // It's an ad-hoc for layout-independent special symbols
    // TODO: doesn't work that way at the moment though but okay-ish
    behavior_config.combo.combos[0] = Some(Combo::new(ComboConfig::new(
        [k!(Kc1), k!(Q)],
        shifted!(Kc1),
        None,
    )));
    behavior_config.combo.combos[1] = Some(Combo::new(ComboConfig::new(
        [k!(Kc2), k!(W)],
        shifted!(Kc2),
        None,
    )));
    behavior_config.combo.combos[2] = Some(Combo::new(ComboConfig::new(
        [k!(Kc3), k!(E)],
        shifted!(Kc3),
        None,
    )));
    behavior_config.combo.combos[3] = Some(Combo::new(ComboConfig::new(
        [k!(Kc4), k!(R)],
        shifted!(Kc4),
        None,
    )));
    behavior_config.combo.combos[4] = Some(Combo::new(ComboConfig::new(
        [k!(Kc5), k!(T)],
        shifted!(Kc5),
        None,
    )));
    behavior_config.combo.combos[5] = Some(Combo::new(ComboConfig::new(
        [k!(Kc6), k!(Y)],
        shifted!(Kc6),
        None,
    )));
    behavior_config.combo.combos[6] = Some(Combo::new(ComboConfig::new(
        [k!(Kc7), k!(U)],
        shifted!(Kc7),
        None,
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

    behavior_config.keyboard_macros = KeyboardMacrosConfig::new(get_macro_sequences());

    behavior_config
}

pub fn get_macro_sequences() -> [u8; MACRO_SPACE_SIZE] {
    define_macro_sequences(&[
        Vec::from_slice(&[
            MacroOperation::Press(HidKeyCode::LGui),
            MacroOperation::Tap(HidKeyCode::C),
            MacroOperation::Release(HidKeyCode::LGui),
        ])
        .expect("too many elements"),
        Vec::from_slice(&[
            MacroOperation::Press(HidKeyCode::LGui),
            MacroOperation::Tap(HidKeyCode::V),
            MacroOperation::Release(HidKeyCode::LGui),
        ])
        .expect("too many elements"),
    ])
}
