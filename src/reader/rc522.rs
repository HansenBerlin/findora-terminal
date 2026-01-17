use super::UidReader;
use rppal::{
    gpio::Gpio,
    spi::{Bus, Mode, SlaveSelect, Spi},
};
use rppal_mfrc522::Mfrc522;
use std::{thread, time::Duration};

pub struct PiRc522 {
    _rst: rppal::gpio::OutputPin,
    spi: Spi,
    dev: Mfrc522,
}

impl PiRc522 {
    pub fn new(rst_bcm: u8) -> Result<Self, Box<dyn std::error::Error>> {
        let mut rst = Gpio::new()?.get(rst_bcm)?.into_output();

        // SS = CE0
        let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode0)?;

        // device uses &mut Spi in its calls; keep spi in struct and pass &mut as needed
        let mut dev = Mfrc522::new(&spi);

        // reset pulse
        rst.set_low();
        thread::sleep(Duration::from_millis(50));
        rst.set_high();
        thread::sleep(Duration::from_millis(50));

        dev.reset()?;

        // quick sanity check (optional, but cheap)
        let _ = dev.version()?;

        Ok(Self { _rst: rst, spi, dev })
    }
}

impl UidReader for PiRc522 {
    fn next_uid(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        loop {
            match self.dev.uid(Duration::from_millis(250)) {
                Ok(uid) => {
                    let u32v = uid.to_u32();
                    return Ok(u32v.to_be_bytes().to_vec());
                }
                Err(_) => thread::sleep(Duration::from_millis(20)),
            }
        }
    }
}
