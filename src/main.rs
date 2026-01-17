mod reader;

use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut r = reader::create()?;

    loop {
        let uid = r.next_uid()?; // raw UID bytes (4 bytes for hw reader; sim can be any)
        let hex = uid.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(":");
        let uuid = Uuid::new_v5(&Uuid::NAMESPACE_OID, &uid);
        println!("{hex} -> {uuid}");
    }
}
