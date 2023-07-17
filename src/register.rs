pub use embedded_hal::blocking::i2c::{Write,WriteRead};
pub trait Register {
    const ADDR: u8;
}

pub trait ReadRegisterI2c: Register {
    fn from_i2c<I2C, E>(i2c: &mut I2C, addr: u8) -> Result<Self, E>
    where
        I2C: WriteRead<Error = E>,
        E: core::fmt::Debug,
        Self: Sized;
}

pub trait WriteRegisterI2c: Register {
    fn to_i2c<I2C, E>(&self, i2c: &mut I2C, addr: u8) -> Result<(), E>
    where
        I2C: Write<Error = E>,
        E: core::fmt::Debug,
        Self: Sized;
}
