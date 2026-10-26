#![no_std]
#![no_main]

//! RMK firmware for the ZSA **Planck EZ Glow** (STM32F303CC + IS31FL3737).
//! Builds with `--features glow`. Same design as the base variant; the
//! lighting task replaces the GPIO-only teeth driver.

#[macro_use]
mod macros;
mod boot;
mod is31fl3737;
mod keymap;
mod rgb;
mod vial;

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::flash::Flash;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::i2c;
use embassy_stm32::peripherals::USB;
use embassy_stm32::rcc;
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::CountingMode;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::{Ch3, Ch4};
use embassy_stm32::usb::{Driver, InterruptHandler};
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
    info!("RMK Planck EZ Glow start!");

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

    // Teeth indicator LEDs on TIM4 PWM (stock firmware drives them hard-PWM,
    // LED_LEVEL cycles their CIE-corrected brightness). Left = PB9 = CH4,
    // right = PB8 = CH3.
    let teeth_pwm = SimplePwm::new(
        p.TIM4,
        None,
        None,
        Some(PwmPin::<_, Ch3>::new(p.PB8, OutputType::PushPull)),
        Some(PwmPin::<_, Ch4>::new(p.PB9, OutputType::PushPull)),
        Hertz(1_000),
        CountingMode::EdgeAlignedUp,
    );
    let teeth = teeth_pwm.split(); // keeps PB8/PB9 driven by TIM4

    // Warm-boot disconnect — ZSA USB bootloader leaves DFU firmware running
    // on jump-to-firmware, keeps its D+ pull-up asserted, and USB peripheral
    // clock-off / PIT-reset does not occur. On this hardware, that means the
    // host continues to see the bootloader's USB and the app's own USB is
    // never even enumerated. Handled by:
    //   1. disabling the USB peripheral clock (prevents the USB peripheral
    //      from applying its own internal pull-up after init),
    //   2. driving PA12 (D+) low as a regular GPIO output,
    //   3. holding both of those while the IS31/I2C keymap init is done,
    //   4. restoring PA12 to input and re-enabling USB clock right before
    //      Driver::new, so SET_ADDRESS timing is within the host's window.
    // (Same warm-boot approach as ZSA Voyager's rmk firmware at
    // https://github.com/jpds/rmk-zsa-voyager/commit/d0ba6765.)
    unsafe {
        // RCC_APB1ENR = 0x4002_101C, USB clock gate = bit 23
        let apb1enr = 0x4002_101C as *mut u32;
        let v = core::ptr::read_volatile(apb1enr);
        core::ptr::write_volatile(apb1enr, v & !(1 << 23));
        // GPIOA_MODER = 0x48000000: PA12 mode at bits 25:24 = 0b01 (output)
        let moder = 0x48000000 as *mut u32;
        let m = core::ptr::read_volatile(moder);
        core::ptr::write_volatile(moder, (m & !(0b11 << 24)) | (0b01 << 24));
        // GPIOA_ODR = 0x48000014: PA12 = 0
        let odr = 0x48000014 as *mut u32;
        let d = core::ptr::read_volatile(odr);
        core::ptr::write_volatile(odr, d & !(1 << 12));
    }
    embassy_time::Timer::after_millis(5).await;

    // IS31FL3737 on I2C1 (SCL = PB6, SDA = PB7), address 0x50 (ADDR to GND).
    let mut i2c_config = i2c::Config::default();
    i2c_config.frequency = Hertz(400_000);
    let led_i2c = i2c::I2c::new_blocking(p.I2C1, p.PB6, p.PB7, i2c_config);

    // IS31FL3737 bring-up happens inline here, before USB (Voyager-style):
    // if the LED controller is unresponsive, the writes are swallowed inside
    // the driver and the keyboard still enumerates and scans. Doing it in
    // main instead of inside the lighting task ensures the executor is never
    // blocked on the I2C bus once the keyboard is live.
    #[cfg(feature = "glow")]
    let led_driver = {
        let mut led_driver = is31fl3737::Is31fl3737::new(led_i2c);
        led_driver.init();
        led_driver
    };

    // USB: full-speed classic peripheral without VBUS sensing (stock hardware).
    // Driver creation is deferred until after keymap init (see warm-boot
    // disconnect above) so that the host's SET_ADDRESS timer starts only
    // once the firmware can respond immediately.

    let (row_pins, col_pins) = config_matrix_pins_stm32!(
        peripherals: p,
        input: [PA10, PA9, PA8, PB15, PC13, PC14, PC15, PA2],
        output: [PB11, PB10, PB2, PB1, PA7, PB0]
    );

    let rmk_config = RmkConfig {
        device_config: DeviceConfig {
            vid: 0x3297,
            pid: 0xC6CF,
            product_name: "Planck EZ Glow",
            manufacturer: "ZSA Technology Labs",
            serial_number: rmk::config::RMK_BUILD_INFO,
        },
        vial_config: VialConfig::new(VIAL_KEYBOARD_ID, VIAL_KEYBOARD_DEF, &[(0, 0), (4, 5)]),
        ..Default::default()
    };

    let mut keymap_data = KeymapData::new(keymap::get_default_keymap());
    let mut behavior_config = BehaviorConfig::default();
    // ZSA tri-layer: LOWER (1) + RAISE (2) activate ADJUST (3).
    behavior_config.tri_layer = Some([1, 2, 3]);
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

    let debouncer = DefaultDebouncer::new();
    let mut matrix = Matrix::<_, _, _, ROW, COL, true>::new(row_pins, col_pins, debouncer);
    let mut keyboard = Keyboard::new(&keymap);
    let host_service = HostService::new(&keymap, &rmk_config);
    // Storage init is done: release the warm-boot disconnect and hand
    // PA12/USB back to the peripheral. The host sees D+ come up only now,
    // so enumeration starts against a device that can respond immediately.
    unsafe {
        // GPIOA_MODER = 0x48000000: restore PA12 mode to input (0b00)
        let moder = 0x48000000 as *mut u32;
        let m = core::ptr::read_volatile(moder);
        core::ptr::write_volatile(moder, m & !(0b11 << 24));
        // Re-enable USB peripheral clock.
        let apb1enr = 0x4002_101C as *mut u32;
        let v = core::ptr::read_volatile(apb1enr);
        core::ptr::write_volatile(apb1enr, v | (1 << 23));
    }
    let mut usb_transport = UsbTransport::new(Driver::new(p.USB, Irqs, p.PA12, p.PA11), rmk_config.device_config).with_host_service(&host_service);

    let lighting_task = rgb::run(led_driver, teeth);

    // Start
    rmk::futures::future::join(run_all!(matrix, storage, usb_transport, keyboard), lighting_task).await;
}
