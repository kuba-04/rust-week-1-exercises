pub mod transaction;

use crate::transaction::{
    hash_raw_transaction, read_amount, read_compact_size, read_script, read_txid, read_u32, Input,
    Output, Transaction,
};

pub fn decode(tx_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    let transaction_bytes = hex::decode(tx_hex.as_str())?;
    let mut bytes_slice = transaction_bytes.as_slice();

    // Read version
    let version = read_u32(&mut bytes_slice)?;

    // Read input count
    let input_count = read_compact_size(&mut bytes_slice)?;

    let mut inputs = vec![];
    for _ in 0..input_count {
        let txid = read_txid(&mut bytes_slice)?;
        let vout = read_u32(&mut bytes_slice)?;
        let script_sig = read_script(&mut bytes_slice)?;
        let sequence = read_u32(&mut bytes_slice)?;

        inputs.push(Input {
            txid,
            vout,
            script_sig,
            sequence,
        });
    }

    let output_count = read_compact_size(&mut bytes_slice)?;
    let mut outputs = vec![];
    for _ in 0..output_count {
        let amount = read_amount(&mut bytes_slice)?;
        let script_pubkey = read_script(&mut bytes_slice)?;

        outputs.push(Output {
            amount,
            script_pubkey,
        })
    }

    let lock_time = read_u32(&mut bytes_slice)?;
    let transaction_id = hash_raw_transaction(&transaction_bytes);

    let tx = Transaction {
        version,
        inputs,
        outputs,
        lock_time,
        transaction_id,
    };

    Ok(serde_json::to_string_pretty(&tx)?)
}
