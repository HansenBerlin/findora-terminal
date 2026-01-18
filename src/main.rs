use linux_embedded_hal::spidev::{SpiModeFlags, SpidevOptions};
use linux_embedded_hal::SpidevDevice;
use mfrc522::comm::blocking::spi::SpiInterface;
use mfrc522::Mfrc522;
use std::time::Duration;
use uuid::Uuid;
use std::env;
use reqwest::Client;
use std::error::Error;
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Serialize)]
pub struct NewGameRequest {
    pub redTeamName: String,
    pub blueTeamName: String,
    pub gameLength: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut spi = SpidevDevice::open("/dev/spidev0.0")?;

    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(1_000_000)
        .mode(SpiModeFlags::SPI_MODE_0)
        .build();
    spi.0.configure(&options)?;

    let itf = SpiInterface::new(spi);
    let mut rfid = Mfrc522::new(itf).init().map_err(|e| format!("{e:?}"))?;

    dotenvy::dotenv().ok();


    let endpoint = env::var("BUTTON_API_ENDPOINT")
        .map_err(|_| "Environment variable BUTTON_API_TEAM_ENDPOINT was not found. ")?;
    let base_url = format!("{endpoint}/api/");
    let health_check = format!("{base_url}health");
    let api_url = format!("{base_url}game/new");

    let client = Client::new();

    client
        .get(health_check)
        .send()
        .await?
        .error_for_status()
        .map_err(|_| "Couldn't reach the server!")?;



    println!("RFID listener started, waiting for cards...");
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
                println!("Card detected! UID: {} UUIDv5: {}", uid_hex, u);
                tokio::time::sleep(Duration::from_millis(400)).await;
                let res = client
                    .post(&api_url)
                    .json(&NewGameRequest {
                        redTeamName: "red".to_lowercase(),
                        blueTeamName: "blue".to_lowercase(),
                        gameLength: 300,
                    })
                    .send().await?;
                match res.error_for_status() {
                    Ok(_) => println!("Server notified successfully."),
                    Err(e) => println!("Failed to notify server: {}", e),
                }
                println!("Notified server of card UID: {}", uid_hex);
            }
            Err(_) => {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
    }
}
