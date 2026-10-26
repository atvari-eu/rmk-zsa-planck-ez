//! "Teeth" LED driver (the two bottom-row LEDs flanking the USB port).
//!
//! Left (PB9) lights on LOWER (layer 1), right (PB8) on RAISE (layer 2) and
//! both on ADJUST (layer 3), matching ZSA firmware behavior.
//!
//! On stock firmware these pins are TIM4 CH4/CH3 with configurable CIE'd
//! brightness levels; this port drives them fully-on via plain GPIO. They
//! remain functional PWM pins, so brightness support can be added later.

use embassy_stm32::gpio::{Level, Output};
use rmk::event::{ActionEvent, EventSubscriber, LayerChangeEvent, SubscribableEvent};
use rmk::embassy_futures::select::{Either, select};
use rmk::types::action::Action;

use crate::boot;

pub async fn run(mut left: Output<'static>, mut right: Output<'static>) -> ! {
    let mut layer_sub = LayerChangeEvent::subscriber();
    let mut action_sub = ActionEvent::subscriber();
    loop {
        let outcome = select(layer_sub.next_event(), action_sub.next_event()).await;
        match outcome {
            Either::First(LayerChangeEvent(layer)) => {
                left.set_level(match layer {
                    1 | 3 => Level::High,
                    _ => Level::Low,
                });
                right.set_level(match layer {
                    2 | 3 => Level::High,
                    _ => Level::Low,
                });
            }
            Either::Second(ActionEvent { action, keyboard_event }) => {
                if Action::User(1) == action && keyboard_event.pressed {
                    // Stock LVL: jump to the ST system-memory ROM DFU.
                    boot::jump_to_dfu();
                }
            }
        }
    }
}
