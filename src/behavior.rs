use rmk::config::BehaviorConfig;
use rmk::keyboard::combo::{Combo, ComboConfig};
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

    behavior_config
}
