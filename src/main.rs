#![no_std]
#![no_main]

//! RMK firmware for the ZSA Planck EZ (base variant, STM32F303CC).
//!
//! Built with the manual API: the keymap lives in `src/keymap.rs`, the
//! physical layout is described by `vial.json`.

#[macro_use]
mod macros;
mod boot;
mod keymap;
mod teeth;
mod vial;

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::flash::Flash;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::peripherals::USB;
use embassy_stm32::rcc;
use embassy_stm32::time::Hertz;
use embassy_stm32::usb::{InterruptHandler, Driver};
use embassy_stm32::{Config, bind_interrupts};
use keymap::{COL, ROW};
use panic_probe as _;
use rmk::config::{BehaviorConfig, DeviceConfig, PositionalConfig, RmkConfig, StorageConfig, VialConfig};
use rmk::debounce::default_debouncer::DefaultDebouncer;
use rmk::host::HostService;
use rmk::keyboard::Keyboard;
use rmk::matrix::Matrix;
use rmk::storage::async_flash_wrapper;
use rmk::usb::UsbTransport;
use rmk::{KeymapData, initialize_keymap_and_storage, run_all};
use vial::{VIAL_KEYBOARD_DEF, VIAL_KEYBOARD_ID};

bind_interrupts!(struct Irqs {
    USB_LP_CAN_RX0 => InterruptHandler<USB>;
});

// cortex-m-rt calls `__pre_init` before RAM init. The handler only touches the
// SRAM magic word and the ROM vector table, so it is safe to tail-branch to it
// from assembly without relying on initialised statics.
core::arch::global_asm!(
    ".global __pre_init",
    ".type __pre_init, %function",
    ".thumb_func",
    "__pre_init:",
    "b {handler}",
    handler = sym crate::boot::stm32_dfu_entry_check,
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("RMK Planck EZ (base) start!");

    // RCC config: 8 MHz HSE -> PLL x9 = 72 MHz sysclk (USB = 72/1.5 = 48 MHz),
    // APB1 36 MHz.
    let mut config = Config::default();
    config.rcc.hse = Some(rcc::Hse {
        freq: Hertz(8_000_000),
        mode: rcc::HseMode::Oscillator,
    });
    config.rcc.pll = Some(rcc::Pll {
        src: rcc::PllSource::HSE,
        prediv: rcc::PllPreDiv::DIV1,
        mul: rcc::PllMul::MUL9,
    });
    config.rcc.sys = rcc::Sysclk::PLL1_P;
    config.rcc.ahb_pre = rcc::AHBPrescaler::DIV1;
    config.rcc.apb1_pre = rcc::APBPrescaler::DIV2;
    config.rcc.apb2_pre = rcc::APBPrescaler::DIV1;
    let p = embassy_stm32::init(config);

    // Left "teeth" LED (TIM4_CH4 in stock firmware).
    let left_teeth = Output::new(p.PB9, Level::Low, Speed::Low);
    // Right "teeth" LED (TIM4_CH3 in stock firmware).
    let right_teeth = Output::new(p.PB8, Level::Low, Speed::Low);
    // Optional rotary encoder. Stock Planck EZ has no encoder, but the boards
    // have pads for one; omitted here by default.
    //
    // To enable it, wire the encoder between PB12/PB13 and GND, then:
    //   let encoder = rmk::input_device::rotary_encoder::RotaryEncoder::with_phase(
    //       Input::new(p.PB12, Pull::Up), Input::new(p.PB13, Pull::Up), DefaultPhase, 0)
    //       .with_debounce(5);
    // and add `encoder` to `run_all!` below. Encoder bindings live in
    // `keymap::get_default_encoder_map()`.

    // USB: full-speed classic peripheral without VBUS sensing (stock hardware).
    // Driver creation is deferred until after keymap/storage init so the host's
    // SET_ADDRESS timer starts only once the firmware can respond.

    // Pin config: rows 0-3 = left half, rows 4-7 = wired right half.
    let (row_pins, col_pins) = config_matrix_pins_stm32!(
        peripherals: p,
        input: [PA10, PA9, PA8, PB15, PC13, PC14, PC15, PA2],
        output: [PB11, PB10, PB2, PB1, PA7, PB0]
    );

    // Keyboard config. VID/PID match ZSA's Planck EZ.
    let rmk_config = RmkConfig {
        device_config: DeviceConfig {
            vid: 0x3297,
            pid: 0xC6CE,
            product_name: "Planck EZ",
            manufacturer: "ZSA Technology Labs",
            serial_number: rmk::config::RMK_BUILD_INFO,
        },
        vial_config: VialConfig::new(VIAL_KEYBOARD_ID, VIAL_KEYBOARD_DEF, &[(0, 0), (4, 5)]),
        ..Default::default()
    };

    // Initialize the storage and keymap
    let mut keymap_data = KeymapData::new(keymap::get_default_keymap());
    let mut behavior_config = BehaviorConfig::default();
    // ZSA tri-layer: LOWER (1) + RAISE (2) activate ADJUST (3).
    behavior_config.tri_layer = Some([1, 2, 3]);
    // Oryx layout config (via the `felschr-keymap` feature): permissiveHold +
    // holdOnOtherKeyPress (QMK 25.0). RMK's morse profile supports one mode;
    // hold-on-other-press dominates, so use it for the default tap-hold profile.
    #[cfg(feature = "felschr-keymap")]
    {
        behavior_config.morse.default_profile = rmk::types::morse::MorseProfile::new(
            Some(false),
            Some(rmk::types::morse::MorseMode::HoldOnOtherPress),
            Some(250u16),
            Some(250u16),
        );
    }
    let per_key_config = PositionalConfig::default();
    // Internal flash for Vial keymap persistence. memory.x reserves the last
    // two 2 KB sectors, so StorageConfig::default() never overlaps firmware.
    let flash = async_flash_wrapper(Flash::new_blocking(p.FLASH));
    let storage_config = StorageConfig::default();
    let (keymap, mut storage) = initialize_keymap_and_storage(
        &mut keymap_data,
        flash,
        &storage_config,
        &mut behavior_config,
        &per_key_config,
    )
    .await;

    // Initialize the matrix + keyboard
    let debouncer = DefaultDebouncer::new();
    let mut matrix = Matrix::<_, _, _, ROW, COL, true>::new(row_pins, col_pins, debouncer);
    let mut keyboard = Keyboard::new(&keymap);
    let host_service = HostService::new(&keymap, &rmk_config);
    // Keymap/storage init is done: bring up USB now so the host enumerates
    // against a firmware that is ready to answer, after any first-boot erase.
    let driver = Driver::new(p.USB, Irqs, p.PA12, p.PA11);
    let mut usb_transport = UsbTransport::new(driver, rmk_config.device_config).with_host_service(&host_service);

    let teeth_task = teeth::run(left_teeth, right_teeth);
    rmk::futures::future::join(
        run_all!(matrix, storage, usb_transport, keyboard),
        teeth_task
    ).await;
}
