#![allow(unused_macros)]
#![allow(unused_imports)]

use core::cell::RefCell;

type Console = arduino_hal::hal::usart::Usart0<arduino_hal::DefaultClock>;
pub(crate) static CONSOLE: avr_device::interrupt::Mutex<RefCell<Option<Console>>> =
    avr_device::interrupt::Mutex::new(RefCell::new(None));

macro_rules! cprint {
    ($($t:tt)*) => {
        avr_device::interrupt::free(
            |cs| {
                if let Some(console) = crate::console::CONSOLE.borrow(cs).borrow_mut().as_mut() {
                    let _ = ufmt::uwrite!(console, $($t)*);
                }
            },
        )
    };
}

pub(crate) use cprint;

macro_rules! cprintln {
    ($($t:tt)*) => {
        avr_device::interrupt::free(
            |cs| {
                if let Some(console) = crate::console::CONSOLE.borrow(cs).borrow_mut().as_mut() {
                    let _ = ufmt::uwriteln!(console, $($t)*);
                }
            },
        )
    };
}

pub(crate) use cprintln;

pub fn put_console(console: Console) {
    avr_device::interrupt::free(|cs| {
        *CONSOLE.borrow(cs).borrow_mut() = Some(console);
    })
}
