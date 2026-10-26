//! Lighting task for the Planck EZ Glow: drives the IS31FL3737 underglow
//! plus the two "teeth" indicator LEDs (TIM4 PWM on PB8/PB9, like stock).
//!
//! - global lighting state: enabled / animation mode (plain, breathe,
//!   rainbow) / hue / brightness — applies on every layer
//! - layer 4 (NAV): only the LEDs under E S D F I J K L light up, white,
//!   scaled by the global brightness; all other LEDs are off. Every other
//!   layer paints the selected animation
//! - light actions: RgbTog, RgbHui/Hud, RgbVai/Vad, RgbMode[Forward|Reverse]
//! - LED_LEVEL (the ADJUST key): cycles indicator brightness 0..=4 stock
//!   style (off, 1/4, 2/4, 3/4, full, CIE-corrected), independent of the
//!   underglow brightness key
//! - teeth: left = LOWER, right = RAISE, both = ADJUST

use defmt::info;
use embassy_stm32::peripherals::TIM4;
use embassy_stm32::timer::simple_pwm::SimplePwmChannels;
use embassy_time::{Duration, Ticker};
use rmk::event::{ActionEvent, EventSubscriber, LayerChangeEvent, SubscribableEvent};
use rmk::embassy_futures::select::{Either3, select3};
use rmk::types::action::{Action, LightAction};

use crate::boot;
use crate::is31fl3737::{Is31fl3737, LED_COUNT, MATRIX_TO_LED};

/// LED indices under the NAV layer's lit keys, derived from the matrix map:
/// E=(0,3), S=(1,2), D=(1,3), F=(1,4), I=(4,2), J=(5,1), K=(5,2), L=(5,3).
const NAV_LED_INDICES: [usize; 8] = [
    MATRIX_TO_LED[0][3] as usize,
    MATRIX_TO_LED[1][2] as usize,
    MATRIX_TO_LED[1][3] as usize,
    MATRIX_TO_LED[1][4] as usize,
    MATRIX_TO_LED[4][2] as usize,
    MATRIX_TO_LED[5][1] as usize,
    MATRIX_TO_LED[5][2] as usize,
    MATRIX_TO_LED[5][3] as usize,
];

/// Animation modes, cycled by `RgbModeForward` (and reverse) on ADJUST.
const MODE_PLAIN: u8 = 0;     // solid color, hue/brightness adjustable
const MODE_BREATHE: u8 = 1;   // solid color with respiring brightness
const MODE_RAINBOW: u8 = 2;   // hue wheel over all LEDs, scrolling
const NUM_MODES: u8 = 3;

/// CIE lightness from ZSA's planck_ez.c (http://jared.geek.nz/2013/feb/linear-led-pwm).
fn cie_lightness(v: u16) -> u16 {
    if v <= 5243 {
        v / 9
    } else {
        let y = (((u32::from(v) + 10486) << 8) / (10486 + 0xFFFF)) as u32;
        let y = (y * y * y) >> 8;
        if y > 0xFFFF { 0xFFFF } else { y as u16 }
    }
}

/// Indicator duty cycle at a stock LED_LEVEL (0..=4): off and CIE-corrected
/// quarters upward; returns the numerator against a 0xFFFF denominator.
fn indicator_fraction(level: u8) -> (u32, u32) {
    if level == 0 {
        return (0, 1);
    }
    (u32::from(cie_lightness((0xFFFFu16 / 4 * u16::from(level)) as u16)), 0xFFFF)
}

pub async fn run(
    mut driver: Is31fl3737<'static>,
    mut teeth: SimplePwmChannels<'static, TIM4>,
) -> ! {
    // Init is performed inline in main before USB starts; the task must not
    // block the executor with I2C bring-up.
    info!("Planck EZ Glow: lighting task running");

    let mut layer_sub = LayerChangeEvent::subscriber();
    let mut action_sub = ActionEvent::subscriber();
    let mut ticker = Ticker::every(Duration::from_millis(33));

    let mut enabled = true;
    let mut mode = MODE_RAINBOW;
    let mut hue: u8 = 96;
    let mut value: u8 = 100; // 10..=100, steps of 10, every layer
    let mut phase: u8 = 0;
    let mut current_layer: u8 = 0;
    // Stock default: full indicator brightness, cycled on LED_LEVEL press.
    let mut indicator_level: u8 = 4;

    // PWM channels must be explicitly enabled (CCER) before duty writes show.
    teeth.ch3.enable();
    teeth.ch4.enable();
    teeth.ch3.set_duty_cycle_fully_off();
    teeth.ch4.set_duty_cycle_fully_off();

    loop {
        match select3(layer_sub.next_event(), action_sub.next_event(), next_tick(&mut ticker)).await {
            Either3::First(LayerChangeEvent(layer)) => current_layer = layer,
            Either3::Second(ActionEvent { action, keyboard_event }) => {
                if !keyboard_event.pressed {
                    continue;
                }
                if action == Action::User(1) {
                    // Stock LVL key: jump to the STM32 system-memory ROM DFU.
                    boot::jump_to_dfu();
                }
                if let Action::Light(light) = action {
                    match light {
                        LightAction::RgbTog => enabled = !enabled,
                        LightAction::RgbHui => hue = hue.wrapping_add(10),
                        LightAction::RgbHud => hue = hue.wrapping_sub(10),
                        LightAction::RgbVai => value = (value as u16 + 10).min(100) as u8,
                        LightAction::RgbVad => value = value.saturating_sub(10).max(10),
                        LightAction::RgbModeForward => mode = (mode + 1) % NUM_MODES,
                        LightAction::RgbModeReverse => mode = mode.wrapping_sub(1) % NUM_MODES,
                        _ => (),
                    }
                }
                if matches!(action, Action::User(0)) {
                    // LED_LEVEL: full -> 3/4 -> 1/2 -> 1/4 -> off -> full...
                    indicator_level = (indicator_level + 1) % 5;
                }
            }
            Either3::Third(()) => {
                // Teeth: left = LOWER, right = RAISE, both = ADJUST, duty by level.
                let (num, den) = indicator_fraction(indicator_level);
                let left_on = matches!(current_layer, 1 | 3);
                let right_on = matches!(current_layer, 2 | 3);
                if left_on && num > 0 {
                    teeth.ch4.set_duty_cycle_fraction(num, den);
                } else {
                    teeth.ch4.set_duty_cycle_fully_off();
                }
                if right_on && num > 0 {
                    teeth.ch3.set_duty_cycle_fraction(num, den);
                } else {
                    teeth.ch3.set_duty_cycle_fully_off();
                }

                if !enabled {
                    driver.set_all(0, 0, 0);
                    driver.flush();
                } else if current_layer == 4 {
                    // NAV: white on the 8 navigation keys, everything else off.
                    driver.set_all(0, 0, 0);
                    let v = (u16::from(value) * 255 / 100) as u8;
                    for &led in &NAV_LED_INDICES {
                        if led < LED_COUNT {
                            driver.set_rgb(led, v, v, v);
                        }
                    }
                    driver.flush();
                } else {
                    match mode {
                        MODE_PLAIN => {
                            let (r, g, b) = Is31fl3737::hsv_to_rgb(hue, 255, value);
                            driver.set_all(r, g, b);
                        }
                        MODE_BREATHE => {
                            phase = phase.wrapping_add(2);
                            // Triangle wave 0..value..0 for a breathing fade.
                            let t = phase as i16;
                            let e = ((t - 128).abs() * value.max(5) as i16 + 128) / 128;
                            let (r, g, b) = Is31fl3737::hsv_to_rgb(hue, 255, e as u8);
                            driver.set_all(r, g, b);
                        }
                        _ => {
                            // Rainbow: per-LED hue offset that scrolls over time.
                            phase = phase.wrapping_add(2);
                            for led in 0..LED_COUNT {
                                let h = phase.wrapping_add((led * 5) as u8);
                                let (r, g, b) = Is31fl3737::hsv_to_rgb(h, 255, value);
                                driver.set_rgb(led, r, g, b);
                            }
                        }
                    }
                    driver.flush();
                }
            }
        }
    }
}

// Tiny wrapper so select3 sees a future borrowing the ticker.
async fn next_tick(t: &mut Ticker) {
    t.next().await
}
