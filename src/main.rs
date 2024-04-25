#![no_std]
#![no_main]
use defmt::unwrap;
/// This example demonstrates how to access a given pin from more than one embassy task
/// The on-board LED is toggled by two tasks with slightly different periods, leading to the
/// apparent duty cycle of the LED increasing, then decreasing, linearly. The phenomenon is similar
/// to interference and the 'beats' you can hear if you play two frequencies close to one another
/// [Link explaining it](https://www.physicsclassroom.com/class/sound/Lesson-3/Interference-and-Beats)
use embassy_executor::Spawner;
use embassy_rp::gpio::{self, Input, Pull};
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    channel::{Channel, Sender},
};
use embassy_time::{with_deadline, Duration, Ticker, Timer};
use gpio::{AnyPin, Level, Output};
use {defmt_rtt as _, panic_probe as _};

enum LedState {
    Toggle,
}

static CHANNEL: Channel<ThreadModeRawMutex, LedState, 64> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Hello world via the traditional BLINK
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, Level::Low);

    loop {
        led.set_high();
        Timer::after_secs(1).await;

        led.set_low();
        Timer::after_secs(1).await;
    }

    /*

    // Basic Button Polling
    let p = embassy_rp::init(Default::default());

    let led = Output::new(AnyPin::from(p.PIN_25), Level::Low);
    let external_led = Output::new(AnyPin::from(p.PIN_0), Level::Low);
    let input_pin = Input::new(AnyPin::from(p.PIN_1), Pull::Down);

    let mut error_out_one = Output::new(AnyPin::from(p.PIN_16), Level::High);
    let mut error_out_two = Output::new(AnyPin::from(p.PIN_17), Level::High);

    match spawner.spawn(poll_btn_toggle_led(input_pin, external_led)) {
        Err(_) => {
            error_out_one.set_high();
        }
        _ => (),
    }

    let dt = 100 * 1_000_000;
    match spawner.spawn(toggle_led_no_static(led, Duration::from_nanos(dt))) {
        Err(_) => {
            error_out_two.set_high();
        }
        _ => (),
    }

    */

    /*
    // Channels Example
    let p = embassy_rp::init(Default::default());
    let external_led_one = Output::new(AnyPin::from(p.PIN_0), Level::Low);
    let external_led_two = Output::new(AnyPin::from(p.PIN_17), Level::Low);
    let external_led_three = Output::new(AnyPin::from(p.PIN_16), Level::Low);
    let mut led_collection: [Output<'static>; 3] =
        [external_led_one, external_led_two, external_led_three];

    unwrap!(spawner.spawn(toggle_led_sequence(
        CHANNEL.sender(),
        Duration::from_nanos(1000 * 1_000_000)
    )));

    let mut index: usize = 0;
    led_collection[index].set_high();
    loop {
        match CHANNEL.receive().await {
            LedState::Toggle => {
                led_collection[index].toggle();
                index = index + 1;
                if index == 3 {
                    index = 0;
                }
                led_collection[index].toggle();
            }
        }
    }
    */

    /*
    // LED With Button And Channels
    let p = embassy_rp::init(Default::default());
    let external_led_one = Output::new(AnyPin::from(p.PIN_0), Level::Low);
    let external_led_two = Output::new(AnyPin::from(p.PIN_17), Level::Low);
    let external_led_three = Output::new(AnyPin::from(p.PIN_16), Level::Low);

    let btn_pin = Input::new(AnyPin::from(p.PIN_1), Pull::Down);

    let mut led_collection: [Output<'static>; 3] =
        [external_led_one, external_led_two, external_led_three];

    unwrap!(spawner.spawn(poll_btn_with_state(
        CHANNEL.sender(),
        btn_pin,
        Duration::from_nanos(1_000_000)
    )));

    let mut index: usize = 0;
    led_collection[index].set_high();
    loop {
        match CHANNEL.receive().await {
            LedState::Toggle => {
                led_collection[index].toggle();
                index = index + 1;
                if index == 3 {
                    index = 0;
                }
                led_collection[index].toggle();
            }
        }
    }
    */
}

// Messaging Tasks

#[embassy_executor::task]
async fn toggle_led_sequence(
    control: Sender<'static, ThreadModeRawMutex, LedState, 64>,
    delay: Duration,
) {
    let mut ticker = Ticker::every(delay);
    loop {
        control.send(LedState::Toggle).await;
        ticker.next().await;
    }
}

// Messaging Tasks With A Button

struct Debouncer<'a> {
    input: Input<'a>,
    debounce: Duration,
}

impl<'a> Debouncer<'a> {
    pub fn new(input: Input<'a>, debounce: Duration) -> Self {
        Self { input, debounce }
    }

    pub async fn debounce(&mut self) -> Level {
        loop {
            let l1 = self.input.get_level();
            self.input.wait_for_any_edge().await;
            Timer::after(self.debounce).await;
            let l2 = self.input.get_level();
            if l1 != l2 {
                break l2;
            }
        }
    }
}

#[embassy_executor::task]
async fn poll_btn_with_state(
    control: Sender<'static, ThreadModeRawMutex, LedState, 64>,
    btn: Input<'static>,
    delay: Duration,
) {
    let mut debounce_btn = Debouncer::new(btn, Duration::from_millis(20));
    let mut ticker = Ticker::every(delay);
    loop {
        // Button down
        debounce_btn.debounce().await;
        control.send(LedState::Toggle).await;
        // Button up
        debounce_btn.debounce().await;
        // Wait for the next poll period
        ticker.next().await;
    }
}

// Basic Button Tasks
#[embassy_executor::task]
async fn poll_btn_toggle_led(btn: Input<'static>, mut led: Output<'static>) {
    let duration = Duration::from_nanos(1_000_000);
    let mut ticker = Ticker::every(duration);
    loop {
        if btn.is_high() {
            led.set_high();
        } else {
            led.set_low();
        }
        ticker.next().await;
    }
}

#[embassy_executor::task]
async fn toggle_led_no_static(mut led: Output<'static>, delay: Duration) {
    let mut ticker = Ticker::every(delay);
    loop {
        led.toggle();
        ticker.next().await;
    }
}
