#![no_main]
#![no_std]

#[allow(unused_extern_crates)] // NOTE(allow) bug rust-lang/rust53964

extern crate cortex_m;

extern crate cortex_m_rt as rt;
extern crate f3;
extern crate panic_itm;

//use cortex_m::iprintln;
use rt::{ entry, ExceptionFrame, exception };
use f3::hal::{ prelude::*, serial::Serial, stm32f30x::{self, USART1, USART2}};
use core::fmt::{self, Write};
use heapless::{ consts, Vec};

macro_rules! uprint {
    ($serial:expr, $($arg:tt)*) => {
        $serial.write_fmt(format_args!($($arg)*)).ok()
    };
}

macro_rules! uprintln {
    ($serial:expr, $fmt:expr) => {
        uprint!($serial, concat!($fmt, "\n"))
    };
    ($serial:expr, $fmt:expr, $($arg:tt)*) => {
        uprint!($serial, concat!($fmt, "\n"), $($arg)*)
    };
}

enum SerialPort {
    U1(&'static mut USART1),
    U2(&'static mut USART2)
}

impl SerialPort {
    fn blocking_read(&self) -> u8 {
        match self {
            SerialPort::U1(usart) => {
                while usart.isr.read().rxne().bit_is_clear() {}
                usart.rdr.read().rdr().bits() as u8
            },
            SerialPort::U2(usart) => {
                while usart.isr.read().rxne().bit_is_clear() {}
                usart.rdr.read().rdr().bits() as u8
            }
        }
    }

    fn read(&self) -> Option<u8> {
        match self {
            SerialPort::U1(usart) => {
                if usart.isr.read().rxne().bit_is_set() {
                    return Some(usart.rdr.read().rdr().bits() as u8)
                }
                None
            },
            SerialPort::U2(usart) => {
                if usart.isr.read().rxne().bit_is_set() {
                    return Some(usart.rdr.read().rdr().bits() as u8)
                }
                None
            }
        }
    }

    fn write(&self, c: &u8) {
        match self {
            SerialPort::U1(usart) => {
                while usart.isr.read().txe().bit_is_clear() {}
                usart.tdr.write(|w| w.tdr().bits(u16::from(*c)));
            },
            SerialPort::U2(usart) => {
                while usart.isr.read().txe().bit_is_clear() {}
                usart.tdr.write(|w| w.tdr().bits(u16::from(*c)));
            }
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match self {
            SerialPort::U1(_) => {
                for c in s.bytes() { self.write(&c); }
                Ok(())
            },
            SerialPort::U2(_) => {
                for c in s.bytes() { self.write(&c); }
                Ok(())
            }
        }
    }
}

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
                    buffer.push(b'\n');
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
                    buffer2.push(b'\n');
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


