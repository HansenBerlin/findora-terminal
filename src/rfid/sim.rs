use crate::rfid::{RfidEvent, RfidReader};
use std::io::{self, Write};

pub struct SimReader {
    counter: u32,
}

impl SimReader {
    pub fn new() -> Self {
        Self { counter: 0 }
    }

    fn next_uid(&mut self) -> Vec<u8> {
        self.counter = self.counter.wrapping_add(1);
        let base = [0xDE, 0xAD, 0xBE, 0xEF];
        let c = self.counter.to_be_bytes();
        vec![base[0] ^ c[0], base[1] ^ c[1], base[2] ^ c[2], base[3] ^ c[3]]
    }
}

impl RfidReader for SimReader {
    fn next_event(&mut self) -> Result<RfidEvent, Box<dyn std::error::Error>> {
        print!("Sim: press ENTER to simulate RFID read... ");
        io::stdout().flush()?;
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        Ok(RfidEvent::Uid(self.next_uid()))
    }
}
