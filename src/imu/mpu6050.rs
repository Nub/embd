use super::{AccelRaw, GyroRaw};
use crate::register::*;
use crate::types::*;

pub use registers::AccelCfg;

#[derive(Copy, Clone, Debug, Default)]
pub struct Config {
    accel: AccelCfg,
}

pub struct Driver<I2C> {
    cfg: Config,
    i2c: I2C,
}

impl<I2C, E> Driver<I2C>
where
    I2C: Write<Error = E> + WriteRead<Error = E>,
    E: core::fmt::Debug,
{
    const ADDR: u8 = 0x68;

    pub fn new(i2c: I2C, cfg: Config) -> Self {
        Self { i2c, cfg }
    }

    pub fn configure(&mut self) -> Result<(), ()> {
        self.cfg
            .accel
            .to_i2c(&mut self.i2c, Self::ADDR)
            .map_err(|e| {
                defmt::error!("{}", defmt::Debug2Format(&e));
                ()
            })
    }

    pub fn check_whoami(&mut self) -> Result<(), ()> {
        let expected = registers::WhoAmI::default();
        registers::WhoAmI::from_i2c(&mut self.i2c, Self::ADDR)
            .map_err(|e| {
                defmt::error!("{}", defmt::Debug2Format(&e));
                ()
            })
            .and_then(|got| if expected == got { Ok(()) } else { Err(()) })
    }
}

impl<I2C, E> GyroRaw<u16> for Driver<I2C>
where
    I2C: Write<Error = E> + WriteRead<Error = E>,
    E: core::fmt::Debug,
{
    fn angular_rate_lsb_ratio(&self) -> (u16, u16) {
        (1, 2000)
    }

    fn angular_rate_raw(&self) -> Vector3<u16> {
        Vector3::new(0, 0, 0)
    }
}

impl<I2C, E> AccelRaw<u16> for Driver<I2C>
where
    I2C: Write<Error = E> + WriteRead<Error = E>,
    E: core::fmt::Debug,
{
    fn acceleration_lsb_ratio(&self) -> (u16, u16) {
        (1, 2000)
    }

    fn acceleration_raw(&mut self) -> Vector3<u16> {
        let accl = registers::Accel::from_i2c(&mut self.i2c, Self::ADDR).unwrap();
        Vector3::new(accl.x, accl.y, accl.z)
    }
}

pub mod registers {
    use super::*;
    use bitvec::prelude::*;
    use byteorder::{BigEndian, ByteOrder};
    use defmt::Format;

    #[derive(Debug, Format, Copy, Clone, PartialEq)]
    pub struct WhoAmI(u8);

    impl Default for WhoAmI {
        fn default() -> Self {
            Self(104)
        }
    }

    impl Register for WhoAmI {
        const ADDR: u8 = 0x75;
    }

    impl ReadRegisterI2c for WhoAmI {
        fn from_i2c<I2C, E>(i2c: &mut I2C, addr: u8) -> Result<Self, E>
        where
            I2C: WriteRead<Error = E>,
            E: core::fmt::Debug,
            Self: Sized,
        {
            let mut buf = [0];
            i2c.write_read(addr, &[Self::ADDR], &mut buf)?;
            Ok(Self(buf[0]))
        }
    }

    #[derive(Debug, Default, Copy, Clone, PartialEq)]
    pub struct Accel {
        pub x: u16,
        pub y: u16,
        pub z: u16,
    }

    impl Register for Accel {
        const ADDR: u8 = 0x3B;
    }

    impl ReadRegisterI2c for Accel {
        fn from_i2c<I2C, E>(i2c: &mut I2C, addr: u8) -> Result<Self, E>
        where
            I2C: WriteRead<Error = E>,
            E: core::fmt::Debug,
            Self: Sized,
        {
            let mut buf = [0; core::mem::size_of::<u16>() * 3];
            i2c.write_read(addr, &[], &mut buf)?;

            Ok(Self {
                x: BigEndian::read_u16(&buf[0..=1]),
                y: BigEndian::read_u16(&buf[2..=3]),
                z: BigEndian::read_u16(&buf[3..=4]),
            })
        }
    }

    #[derive(Debug, Default, Format, Copy, Clone, PartialEq)]
    pub struct AccelCfg {
        pub test_x: bool,
        pub test_y: bool,
        pub test_z: bool,
        pub range: AccelRange,
    }

    #[repr(u8)]
    #[derive(Copy, Clone, Debug, Default, Format, PartialEq)]
    pub enum AccelRange {
        PlusMinus2G,
        PlusMinus4G,
        PlusMinus8G,
        #[default]
        PlusMinus16G = 3,
        Unknown,
    }

    impl Register for AccelCfg {
        const ADDR: u8 = 0x1C;
    }

    impl AccelRange {
        fn new(x: u8) -> Self {
            match x {
                0 => Self::PlusMinus2G,
                1 => Self::PlusMinus4G,
                2 => Self::PlusMinus8G,
                3 => Self::PlusMinus16G,
                _ => Self::Unknown,
            }
        }
    }

    impl ReadRegisterI2c for AccelCfg {
        fn from_i2c<I2C, E>(i2c: &mut I2C, addr: u8) -> Result<Self, E>
        where
            I2C: WriteRead<Error = E>,
            E: core::fmt::Debug,
            Self: Sized,
        {
            let mut buf = [0; core::mem::size_of::<u16>() * 3];
            i2c.write_read(addr, &[], &mut buf)?;
            let bits = buf.view_bits::<Msb0>();

            Ok(Self {
                test_x: bits[7],
                test_y: bits[6],
                test_z: bits[5],
                range: AccelRange::new(bits[4..=3].load_be::<u8>()),
            })
        }
    }

    impl WriteRegisterI2c for AccelCfg {
        fn to_i2c<I2C, E>(&self, i2c: &mut I2C, addr: u8) -> Result<(), E>
        where
            I2C: Write<Error = E>,
            E: core::fmt::Debug,
            Self: Sized,
        {
            let mut buf = 0u8;
            let bits = buf.view_bits_mut::<Msb0>();
            bits.set(7, self.test_x);
            bits.set(6, self.test_y);
            bits.set(5, self.test_z);

            let range = self.range as u8;
            let range = range.view_bits::<Msb0>();
            bits.set(4, range[1]);
            bits.set(3, range[0]);

            let cfg: u8 = bits.load_le();
            defmt::info!("write: AccelCfg: {:b}", cfg);

            i2c.write(addr, &[Self::ADDR, cfg])?;

            Ok(())
        }
    }
}
