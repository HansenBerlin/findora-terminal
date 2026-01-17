pub trait UidReader {
    fn next_uid(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
}

#[cfg(all(target_os = "linux", feature = "hw"))]
mod pi_rc522;

mod sim;

pub fn create() -> Result<Box<dyn UidReader>, Box<dyn std::error::Error>> {
    #[cfg(all(target_os = "linux", feature = "hw"))]
    {
        // Your wiring:
        // SS = CE0 (spidev0.0 / rppal Ss0)
        // RST = GPIO25 (BCM25)
        return Ok(Box::new(pi_rc522::PiRc522::new(25)?));
    }

    #[cfg(not(all(target_os = "linux", feature = "hw")))]
    {
        Ok(Box::new(sim::Sim::new()))
    }
}
