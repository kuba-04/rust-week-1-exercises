use std::io::Read;

// Implement extract_tx_version function below
pub fn extract_tx_version(raw_tx_hex: &str) -> Result<u32, String> {
    let transaction_bytes = hex::decode(raw_tx_hex).map_err(|_x| "Hex decode error")?;
    if transaction_bytes.len() < 8 {
        return Err("Transaction data too short".into());
    }

    let mut bytes_slice = transaction_bytes.as_slice();
    Ok(read_version(&mut bytes_slice))
}

fn read_version(transaction_bytes: &mut &[u8]) -> u32 {
    let mut buffer = [0; 4];
    transaction_bytes.read_exact(&mut buffer).unwrap();

    u32::from_le_bytes(buffer)
}
