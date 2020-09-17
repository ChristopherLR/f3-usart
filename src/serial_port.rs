use core::{fmt, fmt::Arguments};
use f3::hal::stm32f30x::{USART1, USART2};

pub enum SerialPort {
    U1(&'static mut USART1),
    U2(&'static mut USART2),
}

impl SerialPort {
    pub fn blocking_read(&self) -> u8 {
        match self {
            SerialPort::U1(usart) => {
                while usart.isr.read().rxne().bit_is_clear() {}
                usart.rdr.read().rdr().bits() as u8
            }
            SerialPort::U2(usart) => {
                while usart.isr.read().rxne().bit_is_clear() {}
                usart.rdr.read().rdr().bits() as u8
            }
        }
    }

    pub fn read(&self) -> Option<u8> {
        match self {
            SerialPort::U1(usart) => {
                if usart.isr.read().rxne().bit_is_set() {
                    return Some(usart.rdr.read().rdr().bits() as u8);
                }
                None
            }
            SerialPort::U2(usart) => {
                if usart.isr.read().rxne().bit_is_set() {
                    return Some(usart.rdr.read().rdr().bits() as u8);
                }
                None
            }
        }
    }

    pub fn write(&self, c: &u8) {
        match self {
            SerialPort::U1(usart) => {
                while usart.isr.read().txe().bit_is_clear() {}
                usart.tdr.write(|w| w.tdr().bits(u16::from(*c)));
            }
            SerialPort::U2(usart) => {
                while usart.isr.read().txe().bit_is_clear() {}
                usart.tdr.write(|w| w.tdr().bits(u16::from(*c)));
            }
        }
    }

    pub fn write_str(&mut self, s: &str) -> fmt::Result {
        match self {
            SerialPort::U1(_) => {
                for c in s.bytes() {
                    self.write(&c);
                }
                Ok(())
            }
            SerialPort::U2(_) => {
                for c in s.bytes() {
                    self.write(&c);
                }
                Ok(())
            }
        }
    }

    pub fn write_fmt(&mut self, args: Arguments) -> fmt::Result {
        if let Some(s) = args.as_str() {
            self.write_str(s)
        } else {
            Ok(())
        }
    }
}

#[macro_export]
macro_rules! uprint {
    ($serial:expr, $($arg:tt)*) => {
        $serial.write_fmt(format_args!($($arg)*)).ok()
    };
}

#[macro_export]
macro_rules! uprintln {
    ($serial:expr, $fmt:expr) => {
        uprint!($serial, concat!($fmt, "\n"))
    };
    ($serial:expr, $fmt:expr, $($arg:tt)*) => {
        uprint!($serial, concat!($fmt, "\n"), $($arg)*)
    };
}
