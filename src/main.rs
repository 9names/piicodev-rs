//! Demo getting readings from some piicodev sensor boards
#![no_std]
#![no_main]

use bme280::BME280;
use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_time::fixed_point::FixedPoint;
use embedded_time::rate::Extensions;
use mpu6050::Mpu6050;
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use pico as bsp;
// use pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
    I2C,
};
use tmp117::TMP117;
use veml6030::Veml6030;

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    let clocks = init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let i2c = I2C::i2c0(
        pac.I2C0,
        pins.gpio8.into_mode(), // sda
        pins.gpio9.into_mode(), // scl
        400.kHz(),
        &mut pac.RESETS,
        125_000_000.Hz(),
    );

    let bus = shared_bus::BusManagerSimple::new(i2c);

    let mut mpu = Mpu6050::new(bus.acquire_i2c());
    match mpu.init(&mut delay) {
        Ok(_) => println!("MPU6050 initialised successfully"),
        Err(_) => println!("Failed to initialise MPU6050"),
    }

    let mut bme = BME280::new_secondary(bus.acquire_i2c());
    match bme.init(&mut delay) {
        Ok(_) => println!("BME280 initialised successfully"),
        Err(_) => println!("Failed to initialise BME280"),
    }

    let mut veml = Veml6030::new(bus.acquire_i2c(), veml6030::SlaveAddr::default());
    match veml.enable() {
        Ok(_) => println!("VEML6030 initialised successfully"),
        Err(_) => println!("Failed to initialise VEML6030"),
    }

    let mut tmp117 = TMP117::new_default(bus.acquire_i2c());
    // TMP117 doesn't have an init function, so read a sample to check if it's okay
    if let Ok(_temperature) = tmp117.read() {
        println!("TMP117 initialised successfully")
    } else {
        println!("Failed to initialise TMP117");
    }

    loop {
        if let Ok(lux) = veml.read_lux() {
            println!(
                "- VEML6030 ambient light sensor -
    light reading {:?}
",
                lux
            )
        } else {
            println!("VEML6030 sample failed")
        }

        if let Ok(temperature) = tmp117.read() {
            println!(
                "- TMP117 precision temperature sensor -
    temp: {:?}c
",
                temperature
            )
        } else {
            println!("TMP117 sample failed")
        }

        if let Ok(environment) = bme.measure(&mut delay) {
            println!(
                "- BME280 environmental monitor -
    Relative Humidity = {}%
    Temperature = {} deg C
    Pressure = {} pascals
",
                environment.humidity, environment.temperature, environment.pressure
            );
        } else {
            println!("BME280 sample failed")
        }

        if let (Some(rc), Some(temp), Some(gyro), Some(acc)) = (
            mpu.get_acc_angles().ok(),
            mpu.get_temp().ok(),
            mpu.get_gyro().ok(),
            mpu.get_acc().ok(),
        ) {
            println!(
                "- MPU6050 motion sensor -
    roll/pitch: {:?}/{:?}
    temperature: {:?}c
    gyro [{:?}, {:?}, {:?}]
    acc [{:?}, {:?}, {:?}]
",
                rc.x, rc.y, temp, gyro.x, gyro.y, gyro.z, acc.x, acc.y, acc.z
            );
        } else {
            println!("MPU6050 sample failed")
        }

        // Only sample every 10 seconds to avoid spamming the console
        delay.delay_ms(10000);
    }
}

// End of file
