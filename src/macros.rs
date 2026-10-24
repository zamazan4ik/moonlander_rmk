macro_rules! config_matrix_pins_stm32 {
    (peripherals: $p:ident, input: [$($in_pin:ident),* $(,)?], output: [$($out_pin:ident),+ $(,)?]) => {
        {
            let output_pins = [
                $(Output::new($p.$out_pin, embassy_stm32::gpio::Level::Low, embassy_stm32::gpio::Speed::VeryHigh)),+
            ];
            let input_pins = [
                $(Input::new($p.$in_pin, embassy_stm32::gpio::Pull::Down)),*
            ];
            (input_pins, output_pins)
        }
    };
}
