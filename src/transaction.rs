use std::io::{Error, ErrorKind, Read};
use serde::{Serialize, Serializer};
use sha2::{Digest, Sha256};

pub fn hash_raw_transaction(raw_tx: &[u8]) -> Txid {
    let mut hasher = Sha256::new();
    hasher.update(&raw_tx);
    let hash1 = hasher.finalize();

    let mut hasher = Sha256::new();
    hasher.update(&hash1);
    let hash2 = hasher.finalize();

    // we can call into() on a trait that implements From, as into is the reciprocal of From and
    // the lib says that finalize returns the GenericArray
    Txid::from_bytes(hash2.into())
}

pub fn read_txid(transaction_bytes: &mut &[u8]) -> Result<Txid, Error> {
    let mut buffer = [0; 32];
    transaction_bytes.read_exact(&mut buffer)?;
    buffer.reverse();
    Ok(Txid::from_bytes(buffer))
}

pub fn read_script(transaction_bytes: &mut &[u8]) -> Result<String, Error> {
    let script_size = read_compact_size(transaction_bytes)? as usize;
    let mut buffer = vec![0_u8; script_size];
    transaction_bytes.read_exact(&mut buffer)?;
    Ok(hex::encode(buffer))
}

pub fn extract_tx_version(raw_tx_hex: &str) -> Result<u32, Error> {
    let transaction_bytes = hex::decode(raw_tx_hex).map_err(|_x| "Hex decode error");
    let transaction_bytes = match transaction_bytes {
        Ok(tb) => tb,
        Err(e) => return Err(Error::new(ErrorKind::InvalidInput, format!("{:?}", e))),
    };
    if transaction_bytes.len() < 8 {
        return Err(Error::new(ErrorKind::InvalidInput, "Transaction data too short"));
    }

    let mut bytes_slice = transaction_bytes.as_slice();
    read_u32(&mut bytes_slice)
}

pub fn extract_tx_size(raw_tx_hex: &str) -> Result<u64, Error> {
    let transaction_bytes = hex::decode(raw_tx_hex);
    let transaction_bytes = match transaction_bytes {
        Ok(tb) => tb,
        Err(e) => return Err(Error::new(ErrorKind::InvalidInput, format!("{:?}", e))),
    };
    let mut bytes_slice = transaction_bytes.as_slice();

    read_compact_size(&mut bytes_slice)
}

pub fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    let mut compact_size = [0_u8; 1];
    transaction_bytes.read_exact(&mut compact_size)?;

    match compact_size[0] {
        0..=252 => Ok(compact_size[0] as u64),
        253 => {
            let mut buffer = [0; 2];
            transaction_bytes.read_exact(&mut buffer)?;
            Ok(u16::from_le_bytes(buffer) as u64)
        },
        254 => {
            let mut buffer = [0; 4];
            transaction_bytes.read_exact(&mut buffer)?;
            Ok(u32::from_le_bytes(buffer) as u64)
        },
        255 => {
            let mut buffer = [0; 8];
            transaction_bytes.read_exact(&mut buffer)?;
            Ok(u64::from_le_bytes(buffer))
        }
        _ => panic!("Invalid compact size"),
    }
}

pub fn read_u32(transaction_bytes: &mut &[u8]) -> Result<u32, Error> {
    let mut buffer = [0; 4];
    transaction_bytes.read_exact(&mut buffer)?;

    Ok(u32::from_le_bytes(buffer))
}

#[derive(Debug, Serialize)]
pub struct Amount(u64);

impl Amount {
    pub fn from_sat(satoshi: u64) -> Amount {
        Amount(satoshi)
    }
}

pub trait BitcoinValue {
    fn to_btc(&self) -> f64;
}

impl BitcoinValue for Amount {
    fn to_btc(&self) -> f64 {
        self.0 as f64 / 100_000_000.0
    }
}

pub fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, Error> {
    let mut buffer = [0; 8];
    transaction_bytes.read_exact(&mut buffer)?;
    let amount = u64::from_le_bytes(buffer);
    Ok(Amount::from_sat(amount))
}


#[derive(Debug, Serialize)]
pub struct Transaction {
    pub transaction_id: Txid,
    pub version: u32,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub lock_time: u32,
}

#[derive(Debug)]
pub struct Txid([u8; 32]);

impl Txid {
    pub fn from_bytes(bytes: [u8; 32]) -> Txid {
        Txid(bytes)
    }
}

impl Serialize for Txid {
    fn serialize<S:Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut bytes = self.0.clone();
        bytes.reverse();
        serializer.serialize_str(&hex::encode(bytes))
    }
}

#[derive(Debug, Serialize)]
pub struct Input {
    pub txid: Txid,
    pub output_index: u32,
    pub script_sig: String, // Vec<u8>
    pub sequence: u32,
}

#[derive(Debug, Serialize)]
pub struct Output {
    #[serde(serialize_with = "as_btc")]
    pub amount: Amount,
    pub script_pubkey: String,
}

pub fn as_btc<S: Serializer, T: BitcoinValue>(t: &T, s: S) -> Result<S::Ok, S::Error> {
    let btc = t.to_btc();
    s.serialize_f64(btc)
}


#[cfg(test)]
mod tests {
    use std::io::Error;
    use crate::{extract_tx_size, read_compact_size};

    #[test]
    fn test_read_compact_size() -> Result<(), Error> {
        let mut bytes = [1_u8].as_slice();
        let count = read_compact_size(&mut bytes)?;
        assert_eq!(count, 1_u64);

        let mut bytes = [253_u8, 0, 1].as_slice();
        let count = read_compact_size(&mut bytes)?;
        assert_eq!(count, 256_u64);

        let mut bytes = [254_u8, 0, 0, 0, 1].as_slice();
        let count = read_compact_size(&mut bytes)?;
        assert_eq!(count, 256_u64.pow(3));

        let mut bytes = [255_u8, 0, 0, 0, 0, 0, 0, 0, 1].as_slice();
        let count = read_compact_size(&mut bytes)?;
        assert_eq!(count, 256_u64.pow(7));

        // non edge value
        let mut bytes = [255_u8, 1, 1, 0, 0, 0, 0, 0, 1].as_slice();
        let count = read_compact_size(&mut bytes)?;
        assert_eq!(count, 256_u64.pow(7) + 1 + 256_u64);

        // real world scenario with weird tx of 20k txs
        let hex = "fd204e";
        let count = extract_tx_size(hex)?;
        assert_eq!(count, 20_000_u64);

        Ok(())
    }
}
