// Implement extract_tx_version function below
pub fn extract_tx_version(raw_tx_hex: &str) -> Result<u32, String> {
    let transaction_bytes = hex::decode(raw_tx_hex).map_err(|_x| "Hex decode error")?;
    if transaction_bytes.len() < 8 {
        return Err("Transaction data too short".into());
    }

    let version_bytes = &transaction_bytes[..4];
    // either TryFrom or TryInto
    // let version_bytes: Result<[u8; 4], TryFromSliceError> = <[u8; 4]>::try_from(version_bytes);
    let version_bytes: [u8; 4] = version_bytes
        .try_into()
        .map_err(|_| "Invalid version bytes")?;
    let le_bytes = u32::from_le_bytes(version_bytes);

    Ok(le_bytes)
}
