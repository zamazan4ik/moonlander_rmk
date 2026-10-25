use super::is31fl3731;
#[cfg(feature = "palettefx")]
use super::is31fl3731::{LED_COUNT, MoonlanderLayout};
use super::mcp23018::{LED_PORTA, LED_PORTB, SharedI2c};
use embassy_stm32::gpio::Output;
#[cfg(feature = "palettefx")]
use embassy_time::Instant;
use embassy_time::Timer;
use rmk::event::{ActionEvent, EventSubscriber, LayerChangeEvent, SubscribableEvent};
#[cfg(feature = "palettefx")]
use rmk::event::{KeyboardEvent, KeyboardEventPos};
use rmk::types::action::Action;
#[cfg(feature = "palettefx")]
use rmk::types::action::LightAction;
#[cfg(feature = "palettefx")]
use rmk_palettefx::color::Hsv;
#[cfg(feature = "palettefx")]
use rmk_palettefx::effects::{
    FlowState, FrameParams, Pcg32, ReactiveState, RippleState, SparkleState, VortexState, gradient,
};
#[cfg(feature = "palettefx")]
use rmk_palettefx::layout::LedLayout;
#[cfg(feature = "palettefx")]
use rmk_palettefx::palette::{BUILTIN_PALETTES, Palette, id};

/// Six status LEDs: left PB5/PB4/PB3, right GPB7/GPB6/GPA7.
/// GPIO indicators are active high; MCP indicators are active low.
fn apply_led_frame(
    led1: &mut Output<'static>,
    led2: &mut Output<'static>,
    led3: &mut Output<'static>,
    bits: u8,
) {
    led1.set_level(((bits & 1) != 0).into());
    led2.set_level(((bits & 2) != 0).into());
    led3.set_level(((bits & 4) != 0).into());
    let portb = (((bits >> 3) & 1) ^ 1) << 7 | (((bits >> 4) & 1) ^ 1) << 6;
    LED_PORTB.store(portb, core::sync::atomic::Ordering::Relaxed);
    LED_PORTA.store(
        (((bits >> 5) & 1) ^ 1) << 7,
        core::sync::atomic::Ordering::Relaxed,
    );
}

/// Initial brightness. With palettefx this feeds `FrameParams::val`;
/// without it scales the rainbow wheel output.
const RGB_BRIGHTNESS: u8 = 0x30;
#[cfg(feature = "palettefx")]
const MIN_VAL: u8 = 0x08;
/// Step applied by a single `light!(RgbVai/RgbVad)` press.
#[cfg(feature = "palettefx")]
const VAL_STEP: u8 = 0x10;

/// Initial animation speed (FrameParams::speed). Adjusted by
/// `light!(RgbSpi/RgbSpd)` at runtime. 128 was tuned on hardware.
#[cfg(feature = "palettefx")]
const RGB_SPEED: u8 = 128;
#[cfg(feature = "palettefx")]
const SPEED_STEP: u8 = 16;

/// Number of simultaneous key-hits the Reactive effect remembers.
/// Each hit fades out over ~1.3 s at the default speed, so 16 comfortably
/// covers sustained ~8-10 keys-per-second typing.
#[cfg(feature = "palettefx")]
const REACTIVE_HITS: usize = 16;

/// Fresh `Pcg32` state for Ripple. Seeded from the current time so
/// the first drop lands on a different key each time Ripple is cycled to.
#[cfg(feature = "palettefx")]
fn fresh_ripple_rng() -> Pcg32 {
    let seed = 0xDEAD_C0DE_CAFE_F00Du64 ^ Instant::now().as_millis();
    Pcg32::new(seed, 0xA02B_DBF7_BB3C_0A7F)
}

/// Dynamically-switchable rmk-palettefx effect for the base layer.
/// Each variant owns its own state so cycling resets the time phase.
/// Gradient is stateless. Ripple carries a Pcg32 for drop-placement
/// randomness; Reactive carries a ring of recent key hits.
#[cfg(feature = "palettefx")]
enum Effect {
    Gradient,
    Flow(FlowState),
    Vortex(VortexState),
    Sparkle(SparkleState),
    Ripple(RippleState, Pcg32),
    Reactive(ReactiveState<REACTIVE_HITS>),
}

#[cfg(feature = "palettefx")]
impl Effect {
    fn tick(&mut self, params: FrameParams<'_>, out: &mut [Hsv; LED_COUNT]) {
        match self {
            Self::Gradient => gradient(&MoonlanderLayout, params, out),
            Self::Flow(s) => s.tick(&MoonlanderLayout, params, out),
            Self::Vortex(s) => s.tick(&MoonlanderLayout, params, out),
            Self::Sparkle(s) => s.tick(&MoonlanderLayout, params, out),
            Self::Ripple(s, rng) => s.tick_with_rng(rng, &MoonlanderLayout, params, out),
            Self::Reactive(s) => s.tick(&MoonlanderLayout, params, out),
        }
    }

    fn next(&mut self) {
        *self = match self {
            Self::Gradient => Self::Flow(FlowState::new()),
            Self::Flow(_) => Self::Vortex(VortexState::new()),
            Self::Vortex(_) => Self::Sparkle(SparkleState::new()),
            Self::Sparkle(_) => Self::Ripple(RippleState::new(), fresh_ripple_rng()),
            Self::Ripple(_, _) => Self::Reactive(ReactiveState::new()),
            Self::Reactive(_) => Self::Gradient,
        };
    }

    fn prev(&mut self) {
        *self = match self {
            Self::Gradient => Self::Reactive(ReactiveState::new()),
            Self::Reactive(_) => Self::Ripple(RippleState::new(), fresh_ripple_rng()),
            Self::Ripple(_, _) => Self::Sparkle(SparkleState::new()),
            Self::Sparkle(_) => Self::Vortex(VortexState::new()),
            Self::Vortex(_) => Self::Flow(FlowState::new()),
            Self::Flow(_) => Self::Gradient,
        };
    }

    /// Record a key press against the Reactive effect; no-op for every
    /// other variant. Returns `true` iff a hit was actually recorded,
    /// so the caller can skip the follow-up re-tick / flush when it
    /// wouldn't be visible.
    fn record_hit(&mut self, led_idx: usize, timer_ms: u32) -> bool {
        let Self::Reactive(s) = self else {
            return false;
        };
        let (x, y) = MoonlanderLayout.position(led_idx);
        s.record_hit(x, y, timer_ms);
        true
    }
}

/// Paint a non-base layer's solid palette. Base layer (0) is animated
/// in the main tick loop instead of being a static paint.
fn paint_static_layer(rgb: &mut is31fl3731::Rgb, layer: u8) {
    match layer {
        1 => rgb.set_all(0x00, 0x10, 0x40), // symbols/F-keys: cool blue
        2 => rgb.set_all(0x30, 0x00, 0x30), // media/nav: magenta
        _ => rgb.set_all(0x20, 0x20, 0x20),
    }
}

/// Tick the active rmk-palettefx effect with the current palette /
/// brightness / speed, then write the resulting HSV frame into the
/// chip buffers. Called from the layer-0 animation path below and on
/// every RGB state change so the LEDs respond immediately.
#[cfg(feature = "palettefx")]
fn tick_base_effect(
    effect: &mut Effect,
    palette: &Palette,
    val: u8,
    speed: u8,
    frame: &mut [Hsv; LED_COUNT],
    rgb: &mut is31fl3731::Rgb,
) {
    effect.tick(
        FrameParams {
            palette,
            speed,
            sat: 255,
            val,
            timer_ms: Instant::now().as_millis() as u32,
        },
        frame,
    );
    rgb.paint_hsv(frame);
}

/// Show the highest active layer as a mirrored 3-bit value on the six
/// status LEDs, animate the per-key RGB matrix when the base layer
/// is active, and swap to a solid per-layer palette for other layers.
///
/// Boot behavior: a 6x300 ms indicator cascade, then an
/// instant clear before the Flow animation takes over. Runs
/// concurrently with matrix/USB setup, so it never delays boot.
pub async fn layer_indicator(
    led1: &mut Output<'static>,
    led2: &mut Output<'static>,
    led3: &mut Output<'static>,
    i2c: &SharedI2c,
) -> ! {
    use rmk::embassy_futures::select::{Either4, select4};

    let mut layer_sub = LayerChangeEvent::subscriber();
    let mut action_sub = ActionEvent::subscriber();
    // KeyboardEvent feeds the Reactive effect's per-key hit history.
    // It's a no-op for other effects; `record_hit` gates the flush.
    #[cfg(feature = "palettefx")]
    let mut key_sub = KeyboardEvent::subscriber();
    let mut rgb = is31fl3731::Rgb::new();
    #[cfg(feature = "palettefx")]
    let mut effect = Effect::Flow(FlowState::new());
    #[cfg(feature = "palettefx")]
    let mut palette_idx: usize = id::AFTERBURN;
    #[cfg(feature = "palettefx")]
    let mut val: u8 = RGB_BRIGHTNESS;
    #[cfg(feature = "palettefx")]
    let mut speed: u8 = RGB_SPEED;
    #[cfg(feature = "palettefx")]
    let mut hsv_frame = [Hsv::default(); LED_COUNT];
    #[cfg(not(feature = "palettefx"))]
    let mut phase: u8 = 0;

    const BOOT_FRAMES: [u8; 7] = [1, 2, 4, 8, 16, 32, 0];
    let (last, lit_frames) = BOOT_FRAMES.split_last().unwrap();
    for &frame in lit_frames {
        apply_led_frame(led1, led2, led3, frame);
        Timer::after_millis(300).await;
    }
    apply_led_frame(led1, led2, led3, *last);

    let mut layer: u8 = 0;
    #[cfg(feature = "palettefx")]
    tick_base_effect(
        &mut effect,
        BUILTIN_PALETTES[palette_idx],
        val,
        speed,
        &mut hsv_frame,
        &mut rgb,
    );
    #[cfg(not(feature = "palettefx"))]
    paint_static_layer(&mut rgb, layer);
    {
        let mut bus = i2c.lock().await;
        let _ = rgb.flush(&mut bus);
    }

    loop {
        let tick = Timer::after_millis(50);
        match select4(
            tick,
            layer_sub.next_event(),
            action_sub.next_event(),
            #[cfg(feature = "palettefx")]
            key_sub.next_event(),
            #[cfg(not(feature = "palettefx"))]
            core::future::pending::<()>(),
        )
        .await
        {
            Either4::First(_) => {
                #[cfg(feature = "palettefx")]
                if layer == 0 {
                    tick_base_effect(
                        &mut effect,
                        BUILTIN_PALETTES[palette_idx],
                        val,
                        speed,
                        &mut hsv_frame,
                        &mut rgb,
                    );
                    let mut bus = i2c.lock().await;
                    let _ = rgb.flush(&mut bus);
                }
                //#[cfg(not(feature = "palettefx"))]
                //paint_static_layer(&mut rgb, layer);
                //let mut bus = i2c.lock().await;
                //let _ = rgb.flush(&mut bus);
            }
            Either4::Second(event) => {
                layer = event.0;
                apply_led_frame(led1, led2, led3, (layer & 7) | ((layer & 7) << 3));
                #[cfg(feature = "palettefx")]
                if layer == 0 {
                    tick_base_effect(
                        &mut effect,
                        BUILTIN_PALETTES[palette_idx],
                        val,
                        speed,
                        &mut hsv_frame,
                        &mut rgb,
                    );
                } else {
                    paint_static_layer(&mut rgb, layer);
                }

                #[cfg(not(feature = "palettefx"))]
                paint_static_layer(&mut rgb, layer);
                let mut bus = i2c.lock().await;
                let _ = rgb.flush(&mut bus);
            }
            Either4::Third(event) => {
                // User 0 is the board's ROM DFU action. RMK's built-in
                // Bootloader action has no STM32 ROM backend in this version.
                if event.action == Action::User(0) && !event.keyboard_event.pressed {
                    super::boot::request_rom_dfu();
                }
                #[cfg(feature = "palettefx")]
                {
                    let Action::Light(light) = event.action else {
                        continue;
                    };
                    // rmk publishes ActionEvent on both press and release;
                    // acting on either would apply each adjustment twice.
                    // Act on press only.
                    if !event.keyboard_event.pressed {
                        continue;
                    }
                    let n = BUILTIN_PALETTES.len();
                    match light {
                        LightAction::RgbModeForward => effect.next(),
                        LightAction::RgbModeReverse => effect.prev(),
                        LightAction::RgbHui => palette_idx = (palette_idx + 1) % n,
                        LightAction::RgbHud => palette_idx = (palette_idx + n - 1) % n,
                        LightAction::RgbVai => val = val.saturating_add(VAL_STEP),
                        LightAction::RgbVad => val = val.saturating_sub(VAL_STEP).max(MIN_VAL),
                        LightAction::RgbSpi => speed = speed.saturating_add(SPEED_STEP),
                        LightAction::RgbSpd => speed = speed.saturating_sub(SPEED_STEP),
                        _ => continue,
                    }
                    if layer == 0 {
                        tick_base_effect(
                            &mut effect,
                            BUILTIN_PALETTES[palette_idx],
                            val,
                            speed,
                            &mut hsv_frame,
                            &mut rgb,
                        );
                        let mut bus = i2c.lock().await;
                        let _ = rgb.flush(&mut bus);
                    }
                }
            }
            Either4::Fourth(_event) => {
                #[cfg(feature = "palettefx")]
                {
                    // Feed the Reactive effect. Other variants don't care,
                    // and `record_hit` short-circuits to a no-op for them.
                    if !_event.pressed || layer != 0 {
                        continue;
                    }
                    let KeyboardEventPos::Key(pos) = _event.pos else {
                        continue;
                    };
                    let Some(led) = is31fl3731::key_to_led(pos.row, pos.col) else {
                        continue;
                    };
                    let now_ms = Instant::now().as_millis() as u32;
                    effect.record_hit(led as usize, now_ms);
                }
            }
        }
    }
}
