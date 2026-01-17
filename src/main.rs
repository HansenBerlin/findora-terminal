use linux_embedded_hal::spidev::{SpiModeFlags, SpidevOptions};
use linux_embedded_hal::SpidevDevice;
use mfrc522::comm::blocking::spi::SpiInterface;
use mfrc522::Mfrc522;
use std::thread;
use std::time::Duration;
use uuid::Uuid;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut spi = SpidevDevice::open("/dev/spidev0.0")?;

    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(1_000_000)
        .mode(SpiModeFlags::SPI_MODE_0)
        .build();
    spi.0.configure(&options)?;

    let itf = SpiInterface::new(spi);
    let mut rfid = Mfrc522::new(itf).init().map_err(|e| format!("{e:?}"))?;

    let client = reqwest::blocking::Client::new();
    dotenvy::dotenv().ok();

    let endpoint = env::var("BUTTON_API_ENDPOINT")
        .map_err(|_| "Environment variable BUTTON_API_TEAM_ENDPOINT was not found. ")?;
    let api_url = format!("{endpoint}/api/game/new").as_str();

    loop {
        match rfid.new_card_present() {
            Ok(atqa) => {
                let uid = rfid.select(&atqa).map_err(|e| format!("{e:?}"))?;
                let uid_bytes = uid.as_bytes();

                let uid_hex = uid_bytes
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(":");

                let u = Uuid::new_v5(&Uuid::NAMESPACE_OID, uid_bytes);

                println!("{uid_hex} -> {u}");

                let resp = client.post(api_url).send();

                match resp {
                    Ok(r) => {
                        let status = r.status();
                        let body = r.text().unwrap_or_default();
                        if body.is_empty() {
                            println!("{status}");
                        } else {
                            println!("{status} {body}");
                        }
                    }
                    Err(e) => eprintln!("{e}"),
                }

                let _ = rfid.hlta();
                thread::sleep(Duration::from_millis(400));
            }
            Err(_) => {
                thread::sleep(Duration::from_millis(50));
            }
        }
    }
}
