//! Per-key RGB via two IS31FL3731 drivers on the shared I2C bus:
//!   0x74 (ADDR = GND): left half, 36 keys
//!   0x77 (ADDR = VCC): right half, 36 keys
//!
//! The 72-entry `LED_TABLE` maps a flat LED index to (chip, r_reg,
//! g_reg, b_reg). `KEY_TO_LED` maps matrix (row, col) -> LED index or
//! `NO_LED` for cells with no physical LED. Both come straight from
//! the Moonlander Mark I hardware layout.

use embassy_stm32::i2c::{I2c, Master};
use embassy_stm32::mode::Blocking;
#[cfg(feature = "palettefx")]
use rmk_palettefx::color::{Hsv, hsv_to_rgb};
#[cfg(feature = "palettefx")]
use rmk_palettefx::layout::LedLayout;

use crate::keymap::{COL, ROW};

pub const ADDR_LEFT: u8 = 0x74;
pub const ADDR_RIGHT: u8 = 0x77;

const CHIP_COUNT: usize = 2;
const PWM_BYTES: usize = 144;
const LED_CONTROL_COUNT: u8 = 18;
const PWM_CHUNKS: u8 = 9;
const CHUNK_BYTES: usize = 16;

const REG_COMMAND: u8 = 0xFD;
const CMD_FRAME_1: u8 = 0x00;
const CMD_FUNCTION: u8 = 0x0B;

const FN_CONFIG: u8 = 0x00;
const FN_PICTURE_DISPLAY: u8 = 0x01;
const FN_AUDIO_SYNC: u8 = 0x06;
const FN_SHUTDOWN: u8 = 0x0A;
const CONFIG_MODE_PICTURE: u8 = 0x00;

const FRAME_REG_LED_CONTROL: u8 = 0x00;
const FRAME_REG_PWM: u8 = 0x24;

/// Encode the IS31FL3731 "Cn_m" register name: `(n-1)*16 + (m-1)`.
const fn c(n: u8, m: u8) -> u8 {
    (n - 1) * 16 + (m - 1)
}

#[derive(Copy, Clone)]
struct LedEntry {
    chip: u8,
    r: u8,
    g: u8,
    b: u8,
}

const fn e(chip: u8, r: u8, g: u8, b: u8) -> LedEntry {
    LedEntry { chip, r, g, b }
}

/// Number of physical RGB LEDs across both halves.
pub const LED_COUNT: usize = 72;

/// Moonlander IS31FL3731 RGB channels, in QMK's LED order.
/// Source: qmk_firmware/keyboards/zsa/moonlander/moonlander.c.
#[rustfmt::skip]
const LED_TABLE: [LedEntry; LED_COUNT] = [
    e(0, c(3, 2), c(1, 1), c(4, 2)),
    e(0, c(2, 2), c(1, 2), c(4, 3)),
    e(0, c(2, 3), c(1, 3), c(3, 3)),
    e(0, c(2, 4), c(1, 4), c(3, 4)),
    e(0, c(2, 5), c(1, 5), c(3, 5)),
    e(0, c(2, 6), c(1, 6), c(3, 6)),
    e(0, c(2, 7), c(1, 7), c(3, 7)),
    e(0, c(2, 8), c(1, 8), c(3, 8)),
    e(0, c(3, 1), c(2, 1), c(4, 1)),
    e(0, c(7, 8), c(6, 8), c(8, 8)),
    e(0, c(7, 7), c(6, 7), c(9, 8)),
    e(0, c(8, 7), c(6, 6), c(9, 7)),
    e(0, c(8, 6), c(7, 6), c(9, 6)),
    e(0, c(8, 5), c(7, 5), c(9, 5)),
    e(0, c(8, 4), c(7, 4), c(9, 4)),
    e(0, c(8, 3), c(7, 3), c(9, 3)),
    e(0, c(8, 2), c(7, 2), c(9, 2)),
    e(0, c(8, 1), c(7, 1), c(9, 1)),
    e(0, c(3, 10), c(1, 9), c(4, 10)),
    e(0, c(2, 10), c(1, 10), c(4, 11)),
    e(0, c(2, 11), c(1, 11), c(3, 11)),
    e(0, c(2, 12), c(1, 12), c(3, 12)),
    e(0, c(2, 13), c(1, 13), c(3, 13)),
    e(0, c(2, 14), c(1, 14), c(3, 14)),
    e(0, c(2, 15), c(1, 15), c(3, 15)),
    e(0, c(2, 16), c(1, 16), c(3, 16)),
    e(0, c(3, 9), c(2, 9), c(4, 9)),
    e(0, c(7, 16), c(6, 16), c(8, 16)),
    e(0, c(7, 15), c(6, 15), c(9, 16)),
    e(0, c(8, 15), c(6, 14), c(9, 15)),
    e(0, c(8, 10), c(7, 10), c(9, 10)),
    e(0, c(8, 9), c(7, 9), c(9, 9)),
    e(0, c(8, 11), c(7, 11), c(9, 11)),
    e(0, c(8, 12), c(7, 12), c(9, 12)),
    e(0, c(8, 13), c(7, 13), c(9, 13)),
    e(0, c(8, 14), c(7, 14), c(9, 14)),
    e(1, c(3, 2), c(1, 1), c(4, 2)),
    e(1, c(2, 2), c(1, 2), c(4, 3)),
    e(1, c(2, 3), c(1, 3), c(3, 3)),
    e(1, c(2, 4), c(1, 4), c(3, 4)),
    e(1, c(2, 5), c(1, 5), c(3, 5)),
    e(1, c(2, 6), c(1, 6), c(3, 6)),
    e(1, c(2, 7), c(1, 7), c(3, 7)),
    e(1, c(2, 8), c(1, 8), c(3, 8)),
    e(1, c(3, 1), c(2, 1), c(4, 1)),
    e(1, c(7, 8), c(6, 8), c(8, 8)),
    e(1, c(7, 7), c(6, 7), c(9, 8)),
    e(1, c(8, 7), c(6, 6), c(9, 7)),
    e(1, c(8, 6), c(7, 6), c(9, 6)),
    e(1, c(8, 5), c(7, 5), c(9, 5)),
    e(1, c(8, 4), c(7, 4), c(9, 4)),
    e(1, c(8, 3), c(7, 3), c(9, 3)),
    e(1, c(8, 2), c(7, 2), c(9, 2)),
    e(1, c(8, 1), c(7, 1), c(9, 1)),
    e(1, c(3, 10), c(1, 9), c(4, 10)),
    e(1, c(2, 10), c(1, 10), c(4, 11)),
    e(1, c(2, 11), c(1, 11), c(3, 11)),
    e(1, c(2, 12), c(1, 12), c(3, 12)),
    e(1, c(2, 13), c(1, 13), c(3, 13)),
    e(1, c(2, 14), c(1, 14), c(3, 14)),
    e(1, c(2, 15), c(1, 15), c(3, 15)),
    e(1, c(2, 16), c(1, 16), c(3, 16)),
    e(1, c(3, 9), c(2, 9), c(4, 9)),
    e(1, c(7, 16), c(6, 16), c(8, 16)),
    e(1, c(7, 15), c(6, 15), c(9, 16)),
    e(1, c(8, 15), c(6, 14), c(9, 15)),
    e(1, c(8, 10), c(7, 10), c(9, 10)),
    e(1, c(8, 9), c(7, 9), c(9, 9)),
    e(1, c(8, 11), c(7, 11), c(9, 11)),
    e(1, c(8, 12), c(7, 12), c(9, 12)),
    e(1, c(8, 13), c(7, 13), c(9, 13)),
    e(1, c(8, 14), c(7, 14), c(9, 14)),
];

/// Horizontal positions in switch-column units for the fallback rainbow.
#[cfg(not(feature = "palettefx"))]
const LED_X: [u8; LED_COUNT] = [
    0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6,
    6, 6, 7, 7, 15, 15, 15, 15, 15, 14, 14, 14, 14, 14, 13, 13, 13, 13, 13, 12, 12, 12, 12, 12, 11,
    11, 11, 11, 11, 10, 10, 10, 10, 9, 9, 9, 10, 9, 8, 8,
];

/// Physical positions from Moonlander's QMK RGB layout.
/// Both axes use 255/240, preserving aspect ratio for rotations and ripples.
#[cfg(feature = "palettefx")]
const LED_POS: [(u8, u8); LED_COUNT] = [
    (0, 4),
    (0, 21),
    (0, 38),
    (0, 55),
    (0, 72),
    (17, 3),
    (17, 20),
    (17, 37),
    (17, 54),
    (17, 71),
    (34, 1),
    (34, 18),
    (34, 35),
    (34, 52),
    (34, 69),
    (51, 0),
    (51, 17),
    (51, 34),
    (51, 51),
    (51, 68),
    (68, 1),
    (68, 18),
    (68, 35),
    (68, 52),
    (68, 69),
    (85, 3),
    (85, 20),
    (85, 37),
    (85, 54),
    (102, 4),
    (102, 21),
    (102, 38),
    (94, 73),
    (106, 85),
    (119, 97),
    (115, 73),
    (255, 4),
    (255, 21),
    (255, 38),
    (255, 55),
    (255, 72),
    (238, 3),
    (238, 20),
    (238, 37),
    (238, 54),
    (238, 71),
    (221, 1),
    (221, 18),
    (221, 35),
    (221, 52),
    (221, 69),
    (204, 0),
    (204, 17),
    (204, 34),
    (204, 51),
    (204, 68),
    (187, 1),
    (187, 18),
    (187, 35),
    (187, 52),
    (187, 69),
    (170, 3),
    (170, 20),
    (170, 37),
    (170, 54),
    (153, 4),
    (153, 21),
    (153, 38),
    (162, 73),
    (149, 85),
    (136, 97),
    (140, 73),
];

#[cfg(feature = "palettefx")]
pub struct MoonlanderLayout;

#[cfg(feature = "palettefx")]
impl LedLayout for MoonlanderLayout {
    fn count(&self) -> usize {
        LED_COUNT
    }

    fn position(&self, index: usize) -> (u8, u8) {
        LED_POS[index]
    }
}

/// Adafruit-style hue wheel: sweeps red -> green -> blue -> red.
#[cfg(not(feature = "palettefx"))]
fn wheel(pos: u8) -> (u8, u8, u8) {
    if pos < 85 {
        (255 - pos * 3, pos * 3, 0)
    } else if pos < 170 {
        let p = pos - 85;
        (0, 255 - p * 3, p * 3)
    } else {
        let p = pos - 170;
        (p * 3, 0, 255 - p * 3)
    }
}

#[cfg(not(feature = "palettefx"))]
fn scale(c: u8, brightness: u8) -> u8 {
    ((c as u16 * brightness as u16) / 255) as u8
}

/// Sentinel in `KEY_TO_LED` for a matrix cell with no physical LED.
pub const NO_LED: u8 = 0xFF;

/// Matrix coordinates to Moonlander LED indices; holes have no physical switch.
#[rustfmt::skip]
pub const KEY_TO_LED: [[u8; COL]; ROW] = [
    [0, 5, 10, 15, 20, 25, 29],
    [1, 6, 11, 16, 21, 26, 30],
    [2, 7, 12, 17, 22, 27, 31],
    [3, 8, 13, 18, 23, 28, NO_LED],
    [4, 9, 14, 19, 24, NO_LED, NO_LED],
    [32, 33, 34, 35, NO_LED, NO_LED, NO_LED],
    [65, 61, 56, 51, 46, 41, 36],
    [66, 62, 57, 52, 47, 42, 37],
    [67, 63, 58, 53, 48, 43, 38],
    [NO_LED, 64, 59, 54, 49, 44, 39],
    [NO_LED, NO_LED, 60, 55, 50, 45, 40],
    [NO_LED, NO_LED, NO_LED, 71, 70, 69, 68],
];

// Validate the hardware tables at compile time: every LED has one switch,
// and RGB channels never alias another key on the same chip.
const _: () = {
    let mut seen = [false; LED_COUNT];
    let mut row = 0;
    while row < ROW {
        let mut col = 0;
        while col < COL {
            let led = KEY_TO_LED[row][col];
            if led != NO_LED {
                assert!((led as usize) < LED_COUNT);
                assert!(!seen[led as usize]);
                seen[led as usize] = true;
            }
            col += 1;
        }
        row += 1;
    }
    let mut channels = [[false; PWM_BYTES]; CHIP_COUNT];
    let mut led = 0;
    while led < LED_COUNT {
        assert!(seen[led]);
        let entry = LED_TABLE[led];
        assert!((entry.chip as usize) < CHIP_COUNT);
        let rgb = [entry.r, entry.g, entry.b];
        let mut component = 0;
        while component < 3 {
            let register = rgb[component] as usize;
            assert!(register < PWM_BYTES);
            assert!(!channels[entry.chip as usize][register]);
            channels[entry.chip as usize][register] = true;
            component += 1;
        }
        led += 1;
    }
};

/// Look up the LED index for a matrix key press. Returns `None` for
/// out-of-range indices or matrix cells with no LED.
#[cfg(feature = "palettefx")]
pub fn key_to_led(row: u8, col: u8) -> Option<u8> {
    let (r, c) = (row as usize, col as usize);
    if r >= ROW || c >= COL {
        return None;
    }
    match KEY_TO_LED[r][c] {
        NO_LED => None,
        idx => Some(idx),
    }
}

fn write_reg(i2c: &mut I2c<'_, Blocking, Master>, addr: u8, reg: u8, val: u8) -> Result<(), ()> {
    i2c.blocking_write(addr, &[reg, val]).map_err(|_| ())
}

fn select_page(i2c: &mut I2c<'_, Blocking, Master>, addr: u8, page: u8) -> Result<(), ()> {
    write_reg(i2c, addr, REG_COMMAND, page)
}

/// Bring a chip into shutdown so the shared bus can be released back
/// to the caller between the two chips' shutdown commands.
///
/// Both chips are told to shut down first, then a single recovery wait
/// covers both before they are configured (the 10 ms is the chip's
/// startup/settle time, not per-chip).
pub fn shutdown_chip(i2c: &mut I2c<'_, Blocking, Master>, addr: u8) -> Result<(), ()> {
    select_page(i2c, addr, CMD_FUNCTION)?;
    write_reg(i2c, addr, FN_SHUTDOWN, 0x00)
}

/// Configure a chip that has already been shut down (see
/// [`shutdown_chip`]): picture mode, every LED control bit enabled, and
/// the PWM buffer cleared. After this, the chip is ready to receive
/// per-key PWM writes via `Rgb::flush`.
pub fn configure_chip(i2c: &mut I2c<'_, Blocking, Master>, addr: u8) -> Result<(), ()> {
    write_reg(i2c, addr, FN_CONFIG, CONFIG_MODE_PICTURE)?;
    write_reg(i2c, addr, FN_PICTURE_DISPLAY, 0x00)?;
    write_reg(i2c, addr, FN_AUDIO_SYNC, 0x00)?;

    select_page(i2c, addr, CMD_FRAME_1)?;
    // The 18 LED-control registers are contiguous, so a single
    // auto-incrementing write clears them all.
    let mut buf = [0xFFu8; 1 + LED_CONTROL_COUNT as usize];
    buf[0] = FRAME_REG_LED_CONTROL;
    i2c.blocking_write(addr, &buf).map_err(|_| ())?;

    let mut buf = [0u8; 1 + CHUNK_BYTES];
    for chunk in 0..PWM_CHUNKS {
        buf[0] = FRAME_REG_PWM + chunk * CHUNK_BYTES as u8;
        i2c.blocking_write(addr, &buf).map_err(|_| ())?;
    }

    select_page(i2c, addr, CMD_FUNCTION)?;
    write_reg(i2c, addr, FN_SHUTDOWN, 0x01)?;
    select_page(i2c, addr, CMD_FRAME_1)?;
    Ok(())
}

/// In-memory PWM state for both chips; flush to push to hardware.
pub struct Rgb {
    bufs: [[u8; PWM_BYTES]; CHIP_COUNT],
    dirty: [bool; CHIP_COUNT],
}

impl Rgb {
    pub const fn new() -> Self {
        Self {
            bufs: [[0; PWM_BYTES]; CHIP_COUNT],
            dirty: [false; CHIP_COUNT],
        }
    }

    pub fn set_all(&mut self, r: u8, g: u8, b: u8) {
        for entry in &LED_TABLE {
            let chip = entry.chip as usize;
            self.bufs[chip][entry.r as usize] = r;
            self.bufs[chip][entry.g as usize] = g;
            self.bufs[chip][entry.b as usize] = b;
            self.dirty[chip] = true;
        }
    }

    /// Write an rmk-palettefx HSV frame to the chip buffers. `frame[i]`
    /// is the colour for the i-th entry of `LED_TABLE`; each sample is
    /// converted to RGB via the rmk-palettefx spectrum converter.
    /// Global brightness is already baked into the HSV `v` field by
    /// `FrameParams::val`, so no further scaling happens here.
    #[cfg(feature = "palettefx")]
    pub fn paint_hsv(&mut self, frame: &[Hsv; LED_COUNT]) {
        for (idx, entry) in LED_TABLE.iter().enumerate() {
            let rgb = hsv_to_rgb(frame[idx]);
            let chip = entry.chip as usize;
            self.bufs[chip][entry.r as usize] = rgb.r;
            self.bufs[chip][entry.g as usize] = rgb.g;
            self.bufs[chip][entry.b as usize] = rgb.b;
            self.dirty[chip] = true;
        }
    }

    /// Horizontal rainbow: each key's hue is offset by its x position
    /// (8 hue units per column) plus `phase`. `brightness` scales the
    /// whole output.
    #[cfg(not(feature = "palettefx"))]
    pub fn paint_rainbow(&mut self, phase: u8, brightness: u8) {
        for (idx, entry) in LED_TABLE.iter().enumerate() {
            let hue = (LED_X[idx] as u16 * 8 + phase as u16) as u8;
            let (r, g, b) = wheel(hue);
            let chip = entry.chip as usize;
            self.bufs[chip][entry.r as usize] = scale(r, brightness);
            self.bufs[chip][entry.g as usize] = scale(g, brightness);
            self.bufs[chip][entry.b as usize] = scale(b, brightness);
            self.dirty[chip] = true;
        }
    }

    pub fn flush(&mut self, i2c: &mut I2c<'_, Blocking, Master>) -> Result<(), ()> {
        for chip in 0..CHIP_COUNT {
            if !self.dirty[chip] {
                continue;
            }
            let addr = if chip == 0 { ADDR_LEFT } else { ADDR_RIGHT };
            let mut buf = [0u8; 1 + CHUNK_BYTES];
            for chunk in 0..PWM_CHUNKS {
                let base = chunk as usize * CHUNK_BYTES;
                buf[0] = FRAME_REG_PWM + chunk * CHUNK_BYTES as u8;
                buf[1..].copy_from_slice(&self.bufs[chip][base..base + CHUNK_BYTES]);
                i2c.blocking_write(addr, &buf).map_err(|_| ())?;
            }
            self.dirty[chip] = false;
        }
        Ok(())
    }
}
