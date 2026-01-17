use super::UidReader;
use std::io::{self, Read};

pub struct Sim {
    counter: u32,
}

impl Sim {
    pub fn new() -> Self {
        Self { counter: 0 }
    }
}

impl UidReader for Sim {
    fn next_uid(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // "button press" = any key in console
        let mut buf = [0u8; 1];
        let _ = io::stdin().read(&mut buf)?;

        self.counter = self.counter.wrapping_add(1);
        Ok(self.counter.to_be_bytes().to_vec())
    }
}
