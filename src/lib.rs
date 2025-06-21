// Implement extract_tx_version function below
pub fn extract_tx_version(raw_tx_hex: &str) -> Result<u32, String> {
    let transaction_bytes = hex::decode(raw_tx_hex).map_err(|_x| "Hex decode error")?;
    if transaction_bytes.len() < 8 {
        return Err("Transaction data too short".into());
    }

    let version_bytes = transaction_bytes[..4]
        .to_vec()
        .iter()
        .map(|x| x.to_owned() as u32)
        .collect::<Vec<u32>>();
    let first_byte = version_bytes
        .first()
        .expect("First byte not found")
        .to_owned();

    Ok(first_byte)
}
