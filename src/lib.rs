use std::io::Read;
use serde::Serialize;

// Implement extract_tx_version function below
pub fn extract_tx_version(raw_tx_hex: &str) -> Result<u32, String> {
    let transaction_bytes = hex::decode(raw_tx_hex).map_err(|_x| "Hex decode error")?;
    if transaction_bytes.len() < 8 {
        return Err("Transaction data too short".into());
    }

    let mut bytes_slice = transaction_bytes.as_slice();
    Ok(read_u32(&mut bytes_slice))
}

pub fn extract_tx_size(raw_tx_hex: &str) -> u64 {
    let transaction_bytes = hex::decode(raw_tx_hex).unwrap();
    let mut bytes_slice = transaction_bytes.as_slice();

    read_compact_size(&mut bytes_slice)
}

pub fn read_compact_size(transaction_bytes: &mut &[u8]) -> u64 {
    let mut compact_size = [0_u8; 1];
    transaction_bytes.read_exact(&mut compact_size).unwrap();

    match compact_size[0] {
        0..=252 => compact_size[0] as u64,
        253 => {
            let mut buffer = [0; 2];
            transaction_bytes.read_exact(&mut buffer).unwrap();
            u16::from_le_bytes(buffer) as u64
        },
        254 => {
            let mut buffer = [0; 4];
            transaction_bytes.read_exact(&mut buffer).unwrap();
            u32::from_le_bytes(buffer) as u64
        },
        255 => {
            let mut buffer = [0; 8];
            transaction_bytes.read_exact(&mut buffer).unwrap();
            u64::from_le_bytes(buffer)
        }
        _ => panic!("Invalid compact size"),
    }
}

pub fn read_u32(transaction_bytes: &mut &[u8]) -> u32 {
    let mut buffer = [0; 4];
    transaction_bytes.read_exact(&mut buffer).unwrap();

    u32::from_le_bytes(buffer)
}

#[derive(Debug, Serialize)]
pub struct Amount(u64);

impl Amount {
    pub fn to_btc(&self) -> f64 {
        self.0 as f64 / 100_000_000.0
    }
}

pub fn read_amount(transaction_bytes: &mut &[u8]) -> Amount {
    let mut buffer = [0; 8];
    transaction_bytes.read_exact(&mut buffer).unwrap();
    let amount = u64::from_le_bytes(buffer);
    Amount(amount)
}


#[cfg(test)]
mod tests {
    use crate::{extract_tx_size, read_compact_size};

    #[test]
    fn test_read_compact_size() {
        let mut bytes = [1_u8].as_slice();
        let count = read_compact_size(&mut bytes);
        assert_eq!(count, 1_u64);

        let mut bytes = [253_u8, 0, 1].as_slice();
        let count = read_compact_size(&mut bytes);
        assert_eq!(count, 256_u64);

        let mut bytes = [254_u8, 0, 0, 0, 1].as_slice();
        let count = read_compact_size(&mut bytes);
        assert_eq!(count, 256_u64.pow(3));

        let mut bytes = [255_u8, 0, 0, 0, 0, 0, 0, 0, 1].as_slice();
        let count = read_compact_size(&mut bytes);
        assert_eq!(count, 256_u64.pow(7));

        // non edge value
        let mut bytes = [255_u8, 1, 1, 0, 0, 0, 0, 0, 1].as_slice();
        let count = read_compact_size(&mut bytes);
        assert_eq!(count, 256_u64.pow(7) + 1 + 256_u64);

        // real world scenario with weird tx of 20k txs
        let hex = "fd204e";
        let count = extract_tx_size(hex);
        assert_eq!(count, 20_000_u64);
    }
}
