use crate::types::*;
use core::ops;
use nalgebra::Vector3;

pub mod mpu6050;

/// Fetch raw gyro data
pub trait GyroRaw<T> {
    /// Ratio to apply to readings to get angular rate (rad/s)
    /// Using the following (reading * gyro_ratio.0 / gyro_ratio.1)
    /// See `trait Gyro<T>`  
    fn angular_rate_lsb_ratio(&self) -> (T, T);
    fn angular_rate_raw(&self) -> Vector3<T>;
}

/// Fetch data from a gyro into radians/s
pub trait Gyro<T> {
    /// Get the latest angular rate
    fn angular_rate(&self) -> Vector3<T>;
}

impl<X, T> Gyro<T> for X
where
    X: GyroRaw<T>,
    T: ops::Mul<Output = T> + ops::Div<Output = T> + Copy,
{
    fn angular_rate(&self) -> Vector3<T> {
        let (n, d) = self.angular_rate_lsb_ratio();
        let raw = self.angular_rate_raw();
        let scale = |x| (x as T) * (n as T) / (d as T);
        Vector3::new(scale(raw[0]), scale(raw[1]), scale(raw[2]))
    }
}


/// Fetch raw gyro data
pub trait AccelRaw<T> {
    /// Ratio to apply to readings to get angular rate (rad/s)
    /// Using the following (reading * gyro_ratio.0 / gyro_ratio.1)
    /// See `trait Gyro<T>`  
    fn acceleration_lsb_ratio(&self) -> (T, T);
    fn acceleration_raw(&mut self) -> Vector3<T>;
}

/// Fetch data from a gyro into radians/s
pub trait Accel<T> {
    /// Get the latest angular rate
    fn acceleration(&mut self) -> Vector3<T>;
}

impl<X, T> Accel<T> for X
where
    X: AccelRaw<T>,
    T: ops::Mul<Output = T> + ops::Div<Output = T> + Copy,
{
    fn acceleration(&mut self) -> Vector3<T> {
        let (n, d) = self.acceleration_lsb_ratio();
        let raw = self.acceleration_raw();
        let scale = |x| (x as T) * (n as T) / (d as T);
        Vector3::new(scale(raw[0]), scale(raw[1]), scale(raw[2]))
    }
}
