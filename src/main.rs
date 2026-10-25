#![no_std]
#![no_main]

#[macro_use]
mod macros;
mod behavior;
mod boot;
mod is31fl3731;
mod keymap;
mod led;
mod mcp23018;
mod storage_layout;
mod vial;

use core::ptr;

use embassy_executor::Spawner;
use embassy_stm32::flash::Flash;
use embassy_stm32::gpio::{Input, Level, Output, Speed};
use embassy_stm32::i2c::{self, I2c};
use embassy_stm32::peripherals::USB;
use embassy_stm32::time::{Hertz, mhz};
use embassy_stm32::usb::{Driver, InterruptHandler};
use embassy_stm32::{Config, bind_interrupts};
use embassy_sync::mutex::Mutex;

use embassy_time::Timer;

use keymap::{COL, ROW};
use mcp23018::{Mcp23018Matrix, SharedI2c};
use panic_halt as _;
use rmk::config::{DeviceConfig, PositionalConfig, RmkConfig, VialConfig};
use rmk::core_traits::Runnable;
use rmk::debounce::default_debouncer::DefaultDebouncer;
use rmk::futures::future::join4;
use rmk::host::HostService;
use rmk::keyboard::Keyboard;
use rmk::matrix::Matrix;
use rmk::storage::async_flash_wrapper;
use rmk::usb::UsbTransport;
use rmk::watchdog::{WatchdogFeed, WatchdogRunner};
use rmk::{KeymapData, initialize_keymap_and_storage, run_all};
use vial::{VIAL_KEYBOARD_DEF, VIAL_KEYBOARD_ID};

const IWDG_KR: *mut u32 = 0x4000_3000 as *mut u32;
const IWDG_PR: *mut u32 = 0x4000_3004 as *mut u32;
const IWDG_RLR: *mut u32 = 0x4000_3008 as *mut u32;

fn iwdg_start() {
    unsafe {
        ptr::write_volatile(IWDG_KR, 0xCCCC);
        ptr::write_volatile(IWDG_KR, 0x5555);
        ptr::write_volatile(IWDG_PR, 0b110); // /256 → ~125 Hz ticks at LSI 32 kHz
        ptr::write_volatile(IWDG_RLR, 1250); // 1250 / 125 Hz = 10s timeout
    }
}

struct Stm32Iwdg;

impl WatchdogFeed for Stm32Iwdg {
    fn feed(&mut self) {
        unsafe {
            ptr::write_volatile(IWDG_KR, 0xAAAA);
        }
    }
}

bind_interrupts!(struct Irqs {
    USB_LP_CAN_RX0 => InterruptHandler<USB>;
});

/// Physical rows / cols on the left half (direct-GPIO scan).
const LEFT_ROWS: usize = 6;
const LEFT_COLS: usize = 7;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: mhz(8),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll = Some(Pll {
            src: PllSource::HSE,
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL9,
        });
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV2;
        config.rcc.apb2_pre = APBPrescaler::DIV1;
    }
    let p = embassy_stm32::init(config);

    // Status LEDs: PB5/PB4/PB3 = layer bits 0/1/2. Held here so
    // the layer_indicator future (joined below) can drive them.
    let mut led_bit0 = Output::new(p.PB5, Level::Low, Speed::Low);
    let mut led_bit1 = Output::new(p.PB4, Level::Low, Speed::Low);
    let mut led_bit2 = Output::new(p.PB3, Level::Low, Speed::Low);

    // Hold USB disconnected until flash initialization is complete, including
    // when returning from ROM DFU, so enumeration starts only when we are ready.
    unsafe {
        // RCC_APB1ENR = 0x4002_101C, USB clock gate = bit 23
        let apb1enr = 0x4002_101C as *mut u32;
        let v = ptr::read_volatile(apb1enr);
        ptr::write_volatile(apb1enr, v & !(1 << 23));

        // GPIOA_MODER = 0x48000000, PA12 mode in bits 25:24 (0b01 = output)
        let moder = 0x48000000 as *mut u32;
        let m = ptr::read_volatile(moder);
        ptr::write_volatile(moder, (m & !(0b11 << 24)) | (0b01 << 24));

        // GPIOA_ODR = 0x48000014, PA12 = 0
        let odr = 0x48000014 as *mut u32;
        let d = ptr::read_volatile(odr);
        ptr::write_volatile(odr, d & !(1 << 12));
    }
    Timer::after_millis(5).await;
    // Deassert MCP23018 reset (PB8, active LOW) and let the chip settle
    // before the first I2C transaction. Reset recovery is microsecond
    // scale, so 2 ms is ample.
    let _mcp_reset = Output::new(p.PB8, Level::High, Speed::Low);
    Timer::after_millis(2).await;

    // I2C1 on PB6 (SCL) / PB7 (SDA) at 400 kHz, blocking. The bus is
    // shared between the MCP matrix driver (continuous scanning) and
    // the layer indicator's RGB flush path (on layer change) via a
    // NoopRawMutex; the mutex lives in main's stack frame and is
    // referenced by both futures joined below.
    let mut i2c_config = i2c::Config::default();
    i2c_config.frequency = Hertz::khz(400);
    let shared_i2c: SharedI2c = Mutex::new(I2c::new_blocking(p.I2C1, p.PB6, p.PB7, i2c_config));

    // Bring both IS31FL3731 chips out of shutdown. Failures are
    // swallowed; the keyboard still enumerates and scans if RGB is
    // unresponsive.
    {
        let mut bus = shared_i2c.lock().await;
        let _ = is31fl3731::shutdown_chip(&mut bus, is31fl3731::ADDR_LEFT);
        let _ = is31fl3731::shutdown_chip(&mut bus, is31fl3731::ADDR_RIGHT);
        Timer::after_millis(10).await;
        let _ = is31fl3731::configure_chip(&mut bus, is31fl3731::ADDR_LEFT);
        let _ = is31fl3731::configure_chip(&mut bus, is31fl3731::ADDR_RIGHT);
    }

    // Left-half direct-GPIO matrix. Scans rows 0-5 of the 12x7 keymap.
    let (col_pins, row_pins) = config_matrix_pins_stm32!(
        peripherals: p,
        input:  [PA0, PA1, PA2, PA3, PA6, PA7, PB0],
        output: [PB10, PB11, PB12, PB13, PB14, PB15]
    );

    let rmk_config = RmkConfig {
        device_config: DeviceConfig {
            manufacturer: "RMK",
            product_name: "ZSA Moonlander Mark I",
            vid: 0x3297,
            pid: 0x1969,
            serial_number: "vial:f64c2b3c;rmk:0.9.0;moonlander:reva",
            ..Default::default()
        },
        vial_config: VialConfig::new(VIAL_KEYBOARD_ID, VIAL_KEYBOARD_DEF, &[]),
        ..Default::default()
    };

    let mut flash = Flash::new_blocking(p.FLASH);
    let storage_config =
        storage_layout::prepare(&mut flash).expect("cannot prepare layout storage");
    let flash = async_flash_wrapper(flash);

    let mut keymap_data = KeymapData::new(keymap::get_default_keymap());
    let mut behavior_config = behavior::get_default_behavior();

    let per_key_config = PositionalConfig::new(keymap::HAND_MAP);
    let (keymap, mut storage) = initialize_keymap_and_storage(
        &mut keymap_data,
        flash,
        &storage_config,
        &mut behavior_config,
        &per_key_config,
    )
    .await;

    let left_debouncer = DefaultDebouncer::<LEFT_ROWS, LEFT_COLS>::new();
    let mut left_matrix =
        Matrix::<_, _, _, LEFT_ROWS, LEFT_COLS, false>::new(row_pins, col_pins, left_debouncer);

    let right_debouncer = DefaultDebouncer::<ROW, COL>::new();
    let mut right_matrix = Mcp23018Matrix::new(&shared_i2c, right_debouncer);

    let mut keyboard = Keyboard::new(&keymap);
    let host_service = HostService::new(&keymap, &rmk_config);

    // Release D+ and hand USB to the peripheral after storage initialization.
    unsafe {
        // Restore PA12 MODER to input (0b00).
        let moder = 0x48000000 as *mut u32;
        let m = ptr::read_volatile(moder);
        ptr::write_volatile(moder, m & !(0b11 << 24));

        // Re-enable USB clock.
        let apb1enr = 0x4002_101C as *mut u32;
        let v = ptr::read_volatile(apb1enr);
        ptr::write_volatile(apb1enr, v | (1 << 23));
    }
    let driver = Driver::new(p.USB, Irqs, p.PA12, p.PA11);

    iwdg_start();
    let mut watchdog_runner = WatchdogRunner::new(Stm32Iwdg, embassy_time::Duration::from_secs(5));

    let mut usb_transport =
        UsbTransport::new(driver, rmk_config.device_config).with_host_service(&host_service);
    join4(
        run_all!(left_matrix, right_matrix, storage, watchdog_runner),
        led::layer_indicator(&mut led_bit0, &mut led_bit1, &mut led_bit2, &shared_i2c),
        keyboard.run(),
        usb_transport.run(),
    )
    .await;
}
