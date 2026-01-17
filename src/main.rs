mod rfid;

use rfid::{RfidEvent, RfidReader};
use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = rfid::create_default_reader()?;

    loop {
        match reader.next_event()? {
            RfidEvent::Uid(uid_bytes) => {
                let uid_hex = uid_bytes
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(":");

                // Deterministic UUID derived from UID bytes (UUID v5)
                let u = Uuid::new_v5(&Uuid::NAMESPACE_OID, &uid_bytes);
                println!("{uid_hex} -> {u}");
            }
        }
    }
}
