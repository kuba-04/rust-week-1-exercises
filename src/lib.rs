pub mod transaction;

use self::transaction::{Transaction, Decodable};

pub fn decode(tx_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    let transaction_bytes = hex::decode(tx_hex.as_str())?;
    let transaction = Transaction::consensus_decode(&mut transaction_bytes.as_slice())?;

    Ok(serde_json::to_string_pretty(&transaction)?)
}
