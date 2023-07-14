#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use imu_test as _;

#[rtic::app(
    device = stm32_hal2::pac,
    dispatchers = [I2C1_EV]
)]
mod app {
    use imu_test::imu;
    use stm32_hal2::{
        clocks::Clocks,
        gpio::{self, Edge, Pin, PinMode, Port},
        pac::{self, interrupt, TIM5},
        timer::{Timer, TimerInterrupt},
    };

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        imu: imu::State,
        imu_interrupt: Pin,
        timer: Timer<TIM5>,
        led_pin: Pin,
        button_pin: Pin
    }

    #[init]
    fn init(_cx: init::Context) -> (Shared, Local) {
        // TODO: why does take.unwrap() break?
        let dp = unsafe {pac::Peripherals::steal()};
        defmt::info!("Initializing");

        defmt::info!("Configuring clocks");
        let clock_cfg = Clocks::default();
        clock_cfg.setup().unwrap();

        defmt::info!("Configuring gpio");
        let led_pin = Pin::new(Port::A, 5, PinMode::Output);

        let mut imu_interrupt = Pin::new(Port::A, 0, PinMode::Input);
        imu_interrupt.enable_interrupt(Edge::Rising);

        let mut button_pin = Pin::new(Port::C, 13, PinMode::Input);
        button_pin.enable_interrupt(Edge::Rising);

        defmt::info!("Configuring timer");
        let mut timer = Timer::new_tim5(dp.TIM5, 1. , Default::default(), &clock_cfg);
        timer.enable_interrupt(TimerInterrupt::Update);
        timer.enable();

        (
            Shared {},
            Local {
                imu: imu::State::default(),
                imu_interrupt,
                timer,
                led_pin,
                button_pin
            },
        )
    }

    #[task(priority = 1, binds = EXTI0, local = [imu])]
    fn read_imu(cx: read_imu::Context) {
        *cx.local.imu = cx.local.imu.step();
        defmt::info!("Hi {}", cx.local.imu.generation);
    }

    #[task(priority = 1, binds = EXTI15_10, local = [button_pin])]
    fn button(cx: button::Context) {
        if cx.local.button_pin.is_high() {
           defmt::info!("I am a button");
           gpio::clear_exti_interrupt(cx.local.button_pin.pin);
        }
    }

    #[task(priority = 1, binds = TIM5, local = [timer, led_pin])]
    fn blink_led(cx: blink_led::Context) {
        cx.local.timer.clear_interrupt(TimerInterrupt::Update);

        if cx.local.led_pin.is_low() {
            cx.local.led_pin.set_high();
        } else {
            cx.local.led_pin.set_low();
        }
    }
}
