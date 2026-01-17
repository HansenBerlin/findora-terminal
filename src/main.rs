use linux_embedded_hal::spidev::{SpiModeFlags, SpidevOptions};
use linux_embedded_hal::SpidevDevice;
use mfrc522::comm::blocking::spi::SpiInterface;
use mfrc522::Mfrc522;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

const API_URL: &str = "http://127.0.0.1:8000/rfid"; // change

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // SPI device: CE0 => /dev/spidev0.0
    let mut spi = SpidevDevice::open("/dev/spidev0.0")?;

    // RC522 is typically mode 0; start with 1 MHz
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(1_000_000)
        .mode(SpiModeFlags::SPI_MODE_0)
        .build();
    spi.0.configure(&options)?;

    let itf = SpiInterface::new(spi);
    let mut rfid = Mfrc522::new(itf).init().map_err(|e| format!("{e:?}"))?;

    let client = reqwest::Client::new();

    loop {
        match rfid.new_card_present() {
            Ok(atqa) => {
                let uid = match rfid.select(&atqa) {
                    Ok(u) => u,
                    Err(e) => {
                        eprintln!("select error: {e:?}");
                        let _ = rfid.hlta();
                        sleep(Duration::from_millis(200)).await;
                        continue;
                    }
                };

                let uid_bytes = uid.as_bytes();

                let uid_hex = uid_bytes
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(":");

                // Treat UID as UUID only if it is 16 bytes; otherwise skip POST.
                // (Mifare Classic 1K UIDs are typically 4 or 7 bytes, so this will usually skip.)
                let parsed_uuid = match Uuid::from_slice(uid_bytes) {
                    Ok(u) => Some(u),
                    Err(_) => None,
                };

                println!(
                    "{uid_hex} -> {}",
                    parsed_uuid
                        .map(|u| u.to_string())
                        .unwrap_or_else(|| "<not-a-uuid>".to_string())
                );

                if let Some(u) = parsed_uuid {
                    // example async POST (JSON)
                    let resp = client
                        .post(API_URL)
                        .json(&serde_json::json!({ "uuid": u.to_string() }))
                        .send()
                        .await;

                    match resp {
                        Ok(r) => {
                            let status = r.status();

                            // Try to print a response body (best-effort)
                            let body = r.text().await.unwrap_or_default();

                            if body.is_empty() {
                                println!("POST {API_URL} -> {status}");
                            } else {
                                println!("POST {API_URL} -> {status} | {body}");
                            }
                        }
                        Err(e) => {
                            eprintln!("POST error: {e}");
                        }
                    }
                }

                let _ = rfid.hlta();
                sleep(Duration::from_millis(400)).await;
            }
            Err(_) => {
                sleep(Duration::from_millis(50)).await;
            }
        }
    }
}
