pub enum RfidEvent {
    Uid(Vec<u8>),
}

pub trait RfidReader {
    fn next_event(&mut self) -> Result<RfidEvent, Box<dyn std::error::Error>>;
}

#[cfg(all(target_os = "linux", feature = "rc522"))]
mod rc522;

#[cfg(not(all(target_os = "linux", feature = "rc522")))]
mod sim;

#[cfg(all(target_os = "linux", feature = "rc522"))]
pub fn create_default_reader() -> Result<Box<dyn RfidReader>, Box<dyn std::error::Error>> {
    Ok(Box::new(rc522::Rc522Reader::new("/dev/spidev0.0", 1_000_000)?))
}

#[cfg(all(target_os = "linux", feature = "rc522"))]
pub fn create_default_reader() -> Result<Box<dyn RfidReader>, Box<dyn std::error::Error>> {
    Ok(Box::new(rc522::Rc522Reader::new(
        "/dev/spidev0.0",
        1_000_000,
        25, // RST wired to GPIO25
    )?))
}
