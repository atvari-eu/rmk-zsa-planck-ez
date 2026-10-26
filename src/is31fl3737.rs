//! IS31FL3737 LED driver (Planck EZ Glow), adapted from QMK's
//! `drivers/led/issi/is31fl3737.c` and ZSA's planck-ez LED table.
//!
//! I2C address: 0x50 (ADDR pin to GND). Pages: LED control 0x00, PWM 0x01,
//! auto-breath 0x02, function 0x03; page selection goes through the command
//! register (0xFD), unlocked by writing 0xC5 to register 0xFE first.

use defmt::{error, info};
use embassy_stm32::i2c::{I2c, Master};
use embassy_stm32::mode::Blocking;

pub const LED_COUNT: usize = 47;

const I2C_ADDR: u8 = 0x50;

const REG_COMMAND: u8 = 0xFD;
const REG_COMMAND_WRITE_LOCK: u8 = 0xFE;
const COMMAND_WRITE_LOCK_MAGIC: u8 = 0xC5;

const PAGE_LED_CONTROL: u8 = 0x00;
const PAGE_PWM: u8 = 0x01;
const PAGE_FUNCTION: u8 = 0x03;

const FUNCTION_REG_CONFIGURATION: u8 = 0x00;
const FUNCTION_REG_GLOBAL_CURRENT: u8 = 0x01;
const FUNCTION_REG_SW_PULLUP: u8 = 0x0F;
const FUNCTION_REG_CS_PULLDOWN: u8 = 0x10;

const PULL_0_OHM: u8 = 0;
const LED_CONTROL_REGISTER_COUNT: usize = 24;
const PWM_REGISTER_COUNT: usize = 192;

/// PWM register offset of the red channel of every LED (in the PWM page),
/// from ZSA's `glow.c` `g_is31fl3737_leds` table (SW2 = R, SW1 = G, SW3 = B).
/// Register offset per QMK's is31fl3737.h: (SW-1)*16 + CS-1, plus 2 skipped
/// reserved registers per sheet for CS >= 7 (SW1_CS7 = 0x08, not 0x06).
pub const LED_PWM_R: [u8; LED_COUNT] = [
    0x1D, 0x1C, 0x1B, 0x1A, 0x19, 0x18,
    0x7D, 0x7C, 0x7B, 0x7A, 0x79, 0x78,
    0x15, 0x14, 0x13, 0x12, 0x11, 0x10,
    0x75, 0x74, 0x73, 0x72, 0x71, 0x70,
    0x4D, 0x4C, 0x4B, 0x4A, 0x49, 0x48,
    0xAD, 0xAC, 0xAB, 0xAA, 0xA9, 0xA8,
    0x45, 0x44, 0x43, 0x42, 0x41, 0x40,
    0xA5, 0xA4, 0xA3, 0xA2, 0xA1,
];
pub const LED_PWM_G: [u8; LED_COUNT] = [
    0x0D, 0x0C, 0x0B, 0x0A, 0x09, 0x08,
    0x6D, 0x6C, 0x6B, 0x6A, 0x69, 0x68,
    0x05, 0x04, 0x03, 0x02, 0x01, 0x00,
    0x65, 0x64, 0x63, 0x62, 0x61, 0x60,
    0x3D, 0x3C, 0x3B, 0x3A, 0x39, 0x38,
    0x9D, 0x9C, 0x9B, 0x9A, 0x99, 0x98,
    0x35, 0x34, 0x33, 0x32, 0x31, 0x30,
    0x95, 0x94, 0x93, 0x92, 0x91,
];
pub const LED_PWM_B: [u8; LED_COUNT] = [
    0x2D, 0x2C, 0x2B, 0x2A, 0x29, 0x28,
    0x8D, 0x8C, 0x8B, 0x8A, 0x89, 0x88,
    0x25, 0x24, 0x23, 0x22, 0x21, 0x20,
    0x85, 0x84, 0x83, 0x82, 0x81, 0x80,
    0x5D, 0x5C, 0x5B, 0x5A, 0x59, 0x58,
    0xBD, 0xBC, 0xBB, 0xBA, 0xB9, 0xB8,
    0x55, 0x54, 0x53, 0x52, 0x51, 0x50,
    0xB5, 0xB4, 0xB3, 0xB2, 0xB1,
];

/// Matrix position (rows/cols per `keymap.rs`) -> LED index. `NO_LED` marks
/// the one unused cell inherited from the MIT layout (right of the 2u space).
pub const NO_LED: u8 = 255;
pub const MATRIX_TO_LED: [[u8; 6]; 8] = [
    [0, 1, 2, 3, 4, 5],
    [12, 13, 14, 15, 16, 17],
    [24, 25, 26, 27, 28, 29],
    [36, 37, 38, 45, 46, NO_LED],
    [6, 7, 8, 9, 10, 11],
    [18, 19, 20, 21, 22, 23],
    [30, 31, 32, 33, 34, 35],
    [42, 43, 44, 39, 40, 41],
];

pub struct Is31fl3737<'d> {
    i2c: I2c<'d, Blocking, Master>,
    pwm: [u8; PWM_REGISTER_COUNT],
    pwm_dirty: bool,
}

impl<'d> Is31fl3737<'d> {
    pub fn new(i2c: I2c<'d, Blocking, Master>) -> Self {
        Self {
            i2c,
            pwm: [0; PWM_REGISTER_COUNT],
            pwm_dirty: false,
        }
    }

    fn write_register(&mut self, reg: u8, data: u8) {
        if let Err(e) = self.i2c.blocking_write(I2C_ADDR, &[reg, data]) {
            error!("IS31FL3737 i2c write error: {}", e);
        }
    }

    fn select_page(&mut self, page: u8) {
        self.write_register(REG_COMMAND_WRITE_LOCK, COMMAND_WRITE_LOCK_MAGIC);
        self.write_register(REG_COMMAND, page);
    }

    /// Bring the driver up following QMK's init order: clear LED control and
    /// PWM registers while shut down, then disable shutdown last so no LEDs
    /// flash with garbage data.
    pub fn init(&mut self) {
        // Page 0 (LED control): all off.
        self.select_page(PAGE_LED_CONTROL);
        for reg in 0..LED_CONTROL_REGISTER_COUNT as u8 {
            self.write_register(reg, 0x00);
        }

        // Page 1 (PWM): all channels to 0.
        self.select_page(PAGE_PWM);
        self.pwm_fill(0);

        // Page 3 (function): de-ghost resistors, max global current, then wake.
        self.select_page(PAGE_FUNCTION);
        self.write_register(FUNCTION_REG_SW_PULLUP, PULL_0_OHM);
        self.write_register(FUNCTION_REG_CS_PULLDOWN, PULL_0_OHM);
        self.write_register(FUNCTION_REG_GLOBAL_CURRENT, 0xFF);
        // Configuration: SW enable bit 0x01 (sync mode 0), PFS=8.4kHz clock.
        self.write_register(FUNCTION_REG_CONFIGURATION, 0x01);

        // Wait for the device to wake up.
        embassy_time::block_for(embassy_time::Duration::from_millis(10));

        // Enable every channel listed in the LED table on page 0.
        self.select_page(PAGE_LED_CONTROL);
        for reg in 0..LED_CONTROL_REGISTER_COUNT as u8 {
            self.write_register(reg, 0xFF);
        }

        info!("IS31FL3737 initialized");
    }

    fn write_buf(&mut self, first_reg: u8, data: &[u8]) {
        let mut src = [0u8; 33];
        src[0] = first_reg;
        let n = data.len();
        src[1..1 + n].copy_from_slice(data);
        if let Err(e) = self.i2c.blocking_write(I2C_ADDR, &src[..1 + n]) {
            error!("IS31FL3737 i2c write error: {}", e);
        }
    }

    fn pwm_fill(&mut self, v: u8) {
        let buf = [v; 32];
        let mut written = 0;
        while written < PWM_REGISTER_COUNT {
            let n = (PWM_REGISTER_COUNT - written).min(32);
            self.write_buf(written as u8, &buf[..n]);
            written += n;
        }
    }

    /// Flush the PWM page.
    pub fn flush(&mut self) {
        if self.pwm_dirty {
            self.select_page(PAGE_PWM);
            let data = self.pwm;
            let mut written = 0;
            while written < PWM_REGISTER_COUNT {
                let n = (PWM_REGISTER_COUNT - written).min(32);
                self.write_buf(written as u8, &data[written..written + n]);
                written += n;
            }
            self.pwm_dirty = false;
        }
    }

    /// Set a single RGB channel triplet at a PWM register offset.
    pub fn set_rgb(&mut self, led: usize, r: u8, g: u8, b: u8) {
        if led >= LED_COUNT {
            return;
        }
        self.pwm[LED_PWM_R[led] as usize] = r;
        self.pwm[LED_PWM_G[led] as usize] = g;
        self.pwm[LED_PWM_B[led] as usize] = b;
        self.pwm_dirty = true;
    }

    /// Set all LEDs to one color.
    pub fn set_all(&mut self, r: u8, g: u8, b: u8) {
        for led in 0..LED_COUNT {
            self.set_rgb(led, r, g, b);
        }
    }

    /// HSV -> RGB888 (0-255 h, 0-100 s/v). Compact integer approximation.
    pub fn hsv_to_rgb(h: u8, s: u8, v: u8) -> (u8, u8, u8) {
        // Hue in 0-255, saturation/value in 0-100.

        let v: u16 = (u16::from(v) * 255) / 100;
        let s = u16::from(s);
        let h = u16::from(h) % 256;
        let region = h / 43;
        let rem = (h % 43) * 6;
        let p = (v * (255 - s)) / 255;
        let q = (v * (255 - ((s * rem) >> 8))) / 255;
        let t = (v * (255 - ((s * (255 - rem)) >> 8))) / 255;
        match region {
            0 => (v as u8, t as u8, p as u8),
            1 => (q as u8, v as u8, p as u8),
            2 => (p as u8, v as u8, t as u8),
            3 => (p as u8, q as u8, v as u8),
            4 => (t as u8, p as u8, v as u8),
            _ => (v as u8, p as u8, q as u8),
        }
    }
}
