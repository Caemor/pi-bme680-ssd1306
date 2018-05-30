extern crate bme680;

extern crate linux_embedded_hal as hal;
extern crate embedded_hal;

extern crate ssd1306;

extern crate env_logger;
#[macro_use]
extern crate log;

extern crate embedded_graphics;


use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, Line, Rect};
use embedded_graphics::fonts::Font6x8;


use ssd1306::prelude::*;



use bme680::*;
use embedded_hal::blocking::i2c;
use hal::*;

use std::result;
use std::thread;
use std::time::Duration;


fn main()  -> result::Result<
    (),
    Bme680Error<<hal::I2cdev as i2c::Read>::Error, <hal::I2cdev as i2c::Write>::Error>,
> {
    env_logger::init();

    let i2c = I2cdev::new("/dev/i2c-1").expect("i2c1");

    let i2c2 = I2cdev::new("/dev/i2c-1").expect("i2c1 clone");

    let mut dev = Bme680_dev::init(i2c, Delay {}, 0x77, 25)?;

    let mut disp: GraphicsMode<_> = ssd1306::Builder::new()
        .with_i2c_addr(0x78)
        .connect_i2c(i2c2).into();

    let mut sensor_settings: SensorSettings = Default::default();

    sensor_settings.tph_sett.os_hum = Some(OversamplingSetting::OS2x);
    sensor_settings.tph_sett.os_pres = Some(OversamplingSetting::OS4x);
    sensor_settings.tph_sett.os_temp = Some(OversamplingSetting::OS8x);
    sensor_settings.tph_sett.filter = Some(2);

    sensor_settings.gas_sett.run_gas = Some(0x01);
    sensor_settings.gas_sett.heatr_dur = Some(Duration::from_millis(1500));
    sensor_settings.gas_sett.heatr_temp = Some(320);

    let settings_sel = DesiredSensorSettings::OST_SEL | DesiredSensorSettings::OSP_SEL
        | DesiredSensorSettings::OSH_SEL
        | DesiredSensorSettings::GAS_SENSOR_SEL;

    let profile_dur = dev.get_profile_dur(&sensor_settings)?;
    info!("Duration {:?}", profile_dur);
    info!("Setting sensor settings");
    dev.set_sensor_settings(settings_sel, &sensor_settings)?;
    info!("Setting forced power modes");
    dev.set_sensor_mode(PowerMode::ForcedMode)?;

    let sensor_settings = dev.get_sensor_settings(settings_sel);
    info!("Sensor settings: {:?}", sensor_settings);

   
    disp.init().unwrap();
    disp.flush().unwrap();

    disp.draw(Line::new((8, 16 + 16), (8 + 16, 16 + 16), 1).into_iter());
    disp.draw(Line::new((8, 16 + 16), (8 + 8, 16), 1).into_iter());
    disp.draw(Line::new((8 + 16, 16 + 16), (8 + 8, 16), 1).into_iter());

    disp.draw(Rect::new((48, 16), (48 + 16, 16 + 16), 1u8).into_iter());

    disp.draw(Circle::new((96, 16 + 8), 8, 1u8).into_iter());

    disp.flush().unwrap();

    loop {
        thread::sleep(Duration::from_millis(5000));
        let power_mode = dev.get_sensor_mode();
        info!("Sensor power mode: {:?}", power_mode);
        info!("Setting forced power modes");
        dev.set_sensor_mode(PowerMode::ForcedMode)?;
        info!("Retrieving sensor data");
        let (data, _state) = dev.get_sensor_data()?;
        info!("Sensor Data {:?}", data);
        info!("Temperature {}°C", data.temperature_celsius());
        info!("Pressure {}hPa", data.pressure_hpa());
        info!("Humidity {}%", data.humidity_percent());

        disp.clear();

        disp.draw(Font6x8::render_str(&format!("Tmp: {}°C", data.temperature_celsius())).into_iter());
        disp.draw(
            Font6x8::render_str(&format!("Pres: {}hPa", data.pressure_hpa()))
                .translate((0, 16))
                .into_iter(),
        );

        disp.draw(
            Font6x8::render_str(&format!("Hum {}%", data.humidity_percent()))
                .translate((0, 32))
                .into_iter(),
        );

        disp.flush().unwrap();
    }

}
