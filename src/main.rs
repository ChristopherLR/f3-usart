#![no_main]
#![no_std]
#![feature(fmt_as_str)]
#[allow(unused_extern_crates)] // NOTE(allow) bug rust-lang/rust53964
mod serial_port;
use serial_port::*;

extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate f3;
extern crate panic_itm;

use f3::hal::{
    prelude::*,
    serial::Serial,
    stm32f30x::{self, USART1, USART2},
};
use heapless::{consts, Vec};
use rt::{entry, exception, ExceptionFrame};

#[entry]
fn main() -> ! {
    let p = stm32f30x::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();
    let mut gpioc = p.GPIOC.split(&mut rcc.ahb);
    let mut gpioa = p.GPIOA.split(&mut rcc.ahb);

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let tx1 = gpioc.pc4.into_af7(&mut gpioc.moder, &mut gpioc.afrl);
    let rx1 = gpioc.pc5.into_af7(&mut gpioc.moder, &mut gpioc.afrl);
    let tx2 = gpioa.pa2.into_af7(&mut gpioa.moder, &mut gpioa.afrl);
    let rx2 = gpioa.pa3.into_af7(&mut gpioa.moder, &mut gpioa.afrl);

    Serial::usart1(p.USART1, (tx1, rx1), 9_600.bps(), clocks, &mut rcc.apb2);
    Serial::usart2(p.USART2, (tx2, rx2), 9_600.bps(), clocks, &mut rcc.apb1);
    let mut serial = SerialPort::U1(unsafe { &mut *(USART1::ptr() as *mut _) });
    let serial2 = SerialPort::U2(unsafe { &mut *(USART2::ptr() as *mut _) });

    uprintln!(serial, "The answer is  {}", 40 + 2);

    let mut buffer: Vec<u8, consts::U32> = Vec::new();
    let mut buffer2: Vec<u8, consts::U32> = Vec::new();

    loop {
        loop {
            if let Some(byte) = serial.read() {
                if byte == b'\n' {
                    if buffer.push(b'\n').is_err() {
                        uprintln!(serial, "Error: {}", "Error");
                        break;
                    }
                    for c in &buffer {
                        serial2.write(&c);
                        serial.write(&c);
                    }
                    buffer.clear();
                    break;
                } else {
                    if buffer.push(byte).is_err() {
                        uprintln!(serial, "Error: {}", "Error");
                        break;
                    }
                    serial.write(&byte);
                }
            }
            if let Some(byte) = serial2.read() {
                if byte == b'\n' {
                    if buffer2.push(b'\n').is_err() {
                        uprintln!(serial, "Error: {}", "Error");
                        break;
                    }
                    for c in &buffer2 {
                        serial.write(&c);
                    }
                    buffer2.clear();
                    break;
                } else {
                    if buffer2.push(byte).is_err() {
                        uprintln!(serial, "Error: {}", "Error");
                        break;
                    }
                }
            }
        }
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
