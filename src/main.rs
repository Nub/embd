#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use imu_test as _;

#[rtic::app(
    device = imu_test::hal::pac,
)]
mod app {
    use imu_test::hal::{
        gpio::{self, Edge, ExtiPin, GpioExt, Input, Output, Pull, PushPull},
        i2c::{I2c, I2c1},
        pac::{self, TIM5},
        prelude::*,
        timer::{counter::CounterUs, Event},
    };
    use imu_test::imu::{self, Accel, AccelRaw, Gyro, GyroRaw};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        imu: imu::mpu6050::Driver<I2c1>,
        timer: CounterUs<TIM5>,
        led_pin: gpio::PA5<Output<PushPull>>,
        button_pin: gpio::PC13<Input>,
    }

    #[init]
    fn init(_cx: init::Context) -> (Shared, Local) {
        defmt::info!("Initializing");

        // let mut dp = pac::Peripherals::take().unwrap();
        let mut dp = unsafe { pac::Peripherals::steal() };
        let mut syscfg = dp.SYSCFG.constrain();

        defmt::info!("Configuring timer");
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(16.MHz()).pclk1(8.MHz()).freeze();

        let mut timer = dp.TIM5.counter(&clocks);
        timer.start(1.secs()).unwrap();
        timer.listen(Event::Update);

        defmt::info!("Configuring gpio");
        let gpioa = dp.GPIOA.split();
        let gpiob = dp.GPIOB.split();
        let gpioc = dp.GPIOC.split();

        let mut led_pin = gpioa.pa5.into_push_pull_output();
        led_pin.set_high();

        // imu_interrupt.enable_interrupt(Edge::Rising);

        let mut button_pin = gpioc.pc13.into_input();
        button_pin.set_internal_resistor(Pull::Up);
        button_pin.make_interrupt_source(&mut syscfg);
        button_pin.trigger_on_edge(&mut dp.EXTI, Edge::Falling);
        button_pin.enable_interrupt(&mut dp.EXTI);

        defmt::info!("Default button_state.is_high: {}", button_pin.is_high());

        defmt::info!("Configuring imu");
        let i2c1 = I2c::new(dp.I2C1, (gpiob.pb8, gpiob.pb9), 100.kHz(), &clocks);
        let mut imu = imu::mpu6050::Driver::new(i2c1, imu::mpu6050::Config::default());
        imu.check_whoami().expect("No mpu6050 found");
        imu.configure()
            .expect("Failed to configure mpu6050");

        (
            Shared {},
            Local {
                imu,
                // imu_interrupt,
                timer,
                led_pin,
                button_pin,
            },
        )
    }

    #[task(priority = 1, binds = EXTI15_10, local = [button_pin, imu])]
    fn button(cx: button::Context) {
        cx.local.button_pin.clear_interrupt_pending_bit();

        let rate = cx.local.imu.angular_rate_raw();
        defmt::info!("gyro: x: {:?}, y: {:?}, z: {:?}", rate.x, rate.y, rate.z);

        let accl = cx.local.imu.acceleration_raw();
        defmt::info!("accl: x: {:?}, y: {:?}, z: {:?}", accl.x, accl.y, accl.z);
    }

    #[task(priority = 1, binds = TIM5, local = [timer, led_pin])]
    fn blink_led(cx: blink_led::Context) {
        if cx.local.led_pin.is_set_low() {
            cx.local.led_pin.set_high();
        } else {
            cx.local.led_pin.set_low();
        }

        cx.local.timer.wait().unwrap();
    }
}
