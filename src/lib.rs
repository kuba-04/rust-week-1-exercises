// Implement extract_tx_version function below
pub fn extract_tx_version(raw_tx_hex: &str) -> Result<u32, String> {
    let version_chars = raw_tx_hex.get(..8);

    if version_chars.is_none() {
        return Err("Transaction data too short".to_string());
    }

    let version_digits = version_chars
        .unwrap()
        .chars()
        .filter(|x| x.is_ascii_digit())
        .map(|x| x.to_digit(10).unwrap())
        .collect::<Vec<u32>>();

    if version_digits.is_empty() {
        return Err("Hex decode error".to_string());
    }

    let version_chunks = version_digits
        .chunks(2)
        .map(|chunk| chunk.to_vec())
        .collect::<Vec<Vec<u32>>>();

    let mut version: u32 = 0;
    for (index, chunk) in version_chunks.iter().enumerate() {
        let left = chunk.first().unwrap() * 16_u32.pow((index + 1) as u32);
        let right = chunk.last().unwrap() * 16_u32.pow((index) as u32);
        version += left + right;
    }

    Ok(version)
}
