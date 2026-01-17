use crate::rfid::{RfidEvent, RfidReader};
use linux_embedded_hal::spidev::{SpiModeFlags, SpidevOptions};
use linux_embedded_hal::{SpidevDevice, SysfsPin};
use mfrc522::comm::blocking::spi::SpiInterface;
use mfrc522::{Initialized, Mfrc522};
use std::thread;
use std::time::Duration;

pub struct Rc522Reader {
    rfid: Mfrc522<SpiInterface<SpidevDevice, SysfsPin>, Initialized>,
}

impl Rc522Reader {
    pub fn new(spidev_path: &str, max_speed_hz: u32, rst_gpio: u64) -> Result<Self, Box<dyn std::error::Error>> {
        let mut spi = SpidevDevice::open(spidev_path)?;

        let options = SpidevOptions::new()
            .bits_per_word(8)
            .max_speed_hz(max_speed_hz)
            .mode(SpiModeFlags::SPI_MODE_0)
            .build();

        spi.0.configure(&options)?;

        // RC522 reset pin (RST on module) via sysfs GPIO
        let rst = SysfsPin::new(rst_gpio);
        let itf = SpiInterface::new(spi, rst);

        let rfid = Mfrc522::new(itf)
            .init()
            .map_err(|e| format!("{e:?}"))?;

        Ok(Self { rfid })
    }
}

impl RfidReader for Rc522Reader {
    fn next_event(&mut self) -> Result<RfidEvent, Box<dyn std::error::Error>> {
        loop {
            match self.rfid.new_card_present() {
                Ok(atqa) => {
                    let uid = self.rfid.select(&atqa).map_err(|e| format!("{e:?}"))?;
                    let bytes = uid.as_bytes().to_vec();
                    let _ = self.rfid.hlta();
                    thread::sleep(Duration::from_millis(250));
                    return Ok(RfidEvent::Uid(bytes));
                }
                Err(_) => thread::sleep(Duration::from_millis(25)),
            }
        }
    }
}
