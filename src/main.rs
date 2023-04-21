#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use core::mem::MaybeUninit;
// use panic_halt as _;
use stackdump_capture::core::{memory_region::ArrayMemoryRegion, register_data::ArrayRegisterData};

use crate::console::{cprint, cprintln};

pub mod console;

#[inline(never)]
#[avr_device::interrupt(atmega328p)]
fn USART_RX() {
    static mut STACK: MaybeUninit<ArrayMemoryRegion<128>> = core::mem::MaybeUninit::uninit();
    static mut REGISTERS: MaybeUninit<ArrayRegisterData<36, u8>> = core::mem::MaybeUninit::uninit();

    let stack = unsafe { STACK.assume_init_mut() };
    let registers = unsafe { REGISTERS.assume_init_mut() };

    stackdump_capture::avr::capture::<128>(stack, registers);

    cprintln!("Registers:");
    for byte in registers.bytes() {
        cprint!("{:02X} ", byte);
    }
    cprintln!("");
    cprintln!("Stack:");
    for byte in stack.bytes() {
        cprint!("{:02X} ", byte);
    }
    cprintln!("");

    panic!()
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    serial.listen(arduino_hal::hal::usart::Event::RxComplete);
    unsafe {
        avr_device::interrupt::enable();
    }
    console::put_console(serial);

    // Digital pin 13 is also connected to an onboard LED marked "L"
    let mut led = pins.d13.into_output();
    led.set_high();

    loop {
        led.toggle();
        arduino_hal::delay_ms(100);
    }
}

#[cfg(not(doc))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // disable interrupts - firmware has panicked so no ISRs should continue running
    avr_device::interrupt::disable();

    // get the peripherals so we can access serial and the LED.
    //
    // SAFETY: Because main() already has references to the peripherals this is an unsafe
    // operation - but because no other code can run after the panic handler was called,
    // we know it is okay.
    let dp = unsafe { arduino_hal::Peripherals::steal() };
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    // Print out panic location
    ufmt::uwriteln!(&mut serial, "Firmware panic!\r").unwrap();
    if let Some(loc) = info.location() {
        ufmt::uwriteln!(
            &mut serial,
            "  At {}:{}:{}\r",
            loc.file(),
            loc.line(),
            loc.column(),
        )
        .unwrap();
    }

    // Blink LED rapidly
    let mut led = pins.d13.into_output();
    loop {
        led.toggle();
        arduino_hal::delay_ms(100);
    }
}
