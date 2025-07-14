use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use sha2::{Digest, Sha256};
use std::io::{BufRead, Write};
use std::{fmt, io};

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref e) => write!(f, "IO Error: {}", e),
        }
    }
}

impl std::error::Error for Error {}

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

#[derive(Debug)]
pub struct Transaction {
    // pub transaction_id: Txid,
    pub version: u32,
    pub inputs: Vec<TxIn>,
    pub outputs: Vec<TxOut>,
    pub lock_time: u32,
}

impl Transaction {
    pub fn compute_txid(&self) -> Txid {
        let mut txid_data = Vec::new();
        self.version
            .consensus_encode(&mut txid_data)
            .expect("writing to a vec shouldn't fail");
        self.inputs
            .consensus_encode(&mut txid_data)
            .expect("writing to a vec shouldn't fail");
        self.outputs
            .consensus_encode(&mut txid_data)
            .expect("writing to a vec shouldn't fail");
        self.lock_time.consensus_encode(&mut txid_data).unwrap();
        Txid::from_raw_transaction(txid_data)
    }
}

impl Serialize for Transaction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tx = serializer.serialize_struct("Transaction", 5)?;
        tx.serialize_field("transaction_id", &self.compute_txid())?;
        tx.serialize_field("version", &self.version)?;
        tx.serialize_field("inputs", &self.inputs)?;
        tx.serialize_field("outputs", &self.outputs)?;
        tx.serialize_field("lock_time", &self.lock_time)?;
        tx.end()
    }
}

#[derive(Debug)]
pub struct Txid([u8; 32]);

impl Txid {
    pub fn from_raw_transaction(data: Vec<u8>) -> Txid {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash1 = hasher.finalize();

        let mut hasher = Sha256::new();
        hasher.update(hash1);
        let mut hash2: [u8; 32] = hasher.finalize().into();

        // Bitcoin displays txids in reverse byte order
        hash2.reverse();
        Txid::from_hash(hash2)
    }
    pub fn from_hash(bytes: [u8; 32]) -> Txid {
        Txid(bytes)
    }
}

impl Serialize for Txid {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Txids are already in display order (big-endian), just encode them
        serializer.serialize_str(&hex::encode(self.0))
    }
}

#[derive(Debug, Serialize)]
pub struct TxIn {
    pub txid: Txid,
    pub vout: u32,
    pub script_sig: String, // Vec<u8>
    pub sequence: u32,
}

#[derive(Debug, Serialize)]
pub struct TxOut {
    #[serde(serialize_with = "as_btc")]
    pub amount: Amount,
    pub script_pubkey: String,
}

pub fn as_btc<S: Serializer, T: BitcoinValue>(t: &T, s: S) -> Result<S::Ok, S::Error> {
    let btc = t.to_btc();
    s.serialize_f64(btc)
}

#[derive(Debug, Serialize)]
pub struct CompactSize(pub u64);

pub trait Decodable: Sized {
    fn consensus_decode<R: BufRead + ?Sized>(r: &mut R) -> Result<Self, io::Error>;
}

pub trait Encodable {
    fn consensus_encode<W: io::Write>(&self, w: &mut W) -> Result<usize, Error>;
}

impl Encodable for u64 {
    fn consensus_encode<W: Write>(&self, w: &mut W) -> Result<usize, Error> {
        let b = self.to_le_bytes();
        let len = w.write(b.as_slice()).map_err(Error::Io)?;
        Ok(len)
    }
}

impl Encodable for u32 {
    fn consensus_encode<W: Write>(&self, w: &mut W) -> Result<usize, Error> {
        let b = self.to_le_bytes();
        let len = w.write(b.as_slice()).map_err(Error::Io)?;
        Ok(len)
    }
}

impl Encodable for u16 {
    fn consensus_encode<W: Write>(&self, w: &mut W) -> Result<usize, Error> {
        let b = self.to_le_bytes();
        let len = w.write(b.as_slice()).map_err(Error::Io)?;
        Ok(len)
    }
}

impl Encodable for u8 {
    fn consensus_encode<W: Write>(&self, w: &mut W) -> Result<usize, Error> {
        let b = self.to_le_bytes();
        let len = w.write(b.as_slice()).map_err(Error::Io)?;
        Ok(len)
    }
}

impl Encodable for [u8; 32] {
    fn consensus_encode<W: Write>(&self, w: &mut W) -> Result<usize, Error> {
        let len = w.write(self.as_slice()).map_err(Error::Io)?;
        Ok(len)
    }
}

impl Encodable for String {
    fn consensus_encode<W: Write>(&self, w: &mut W) -> Result<usize, Error> {
        let b = hex::decode(self).expect("should be a valid hex string");
        let compact_size_len = CompactSize(b.len() as u64).consensus_encode(w)?;
        let b_len = w.write(&b).map_err(Error::Io)?;
        Ok(compact_size_len + b_len)
    }
}

impl Encodable for CompactSize {
    fn consensus_encode<W: Write>(&self, w: &mut W) -> Result<usize, Error> {
        match self.0 {
            0..=0xFC => {
                (self.0 as u8).consensus_encode(w)?;
                Ok(1)
            }
            0xFD..=0xFFFF => {
                w.write([0xFD].as_slice()).map_err(Error::Io)?;
                (self.0 as u16).consensus_encode(w)?;
                Ok(3)
            }
            0x10000..=0xFFFFFFFF => {
                w.write([0xFE].as_slice()).map_err(Error::Io)?;
                (self.0 as u32).consensus_encode(w)?;
                Ok(5)
            }
            _ => {
                w.write([0xFF].as_slice()).map_err(Error::Io)?;
                self.0.consensus_encode(w)?;
                Ok(9)
            }
        }
    }
}

impl Encodable for Vec<TxIn> {
    fn consensus_encode<W: Write>(&self, w: &mut W) -> Result<usize, Error> {
        let mut len = 0;
        len += CompactSize(self.len() as u64).consensus_encode(w)?;
        for tx in self.iter() {
            len += tx.consensus_encode(w)?;
        }
        Ok(len)
    }
}

impl Encodable for Txid {
    fn consensus_encode<W: Write>(&self, w: &mut W) -> Result<usize, Error> {
        // Reverse the bytes back to get the original order for consensus encoding
        let mut bytes = self.0;
        bytes.reverse();
        Ok(bytes.consensus_encode(w)?)
    }
}

impl Encodable for TxIn {
    fn consensus_encode<W: Write>(&self, w: &mut W) -> Result<usize, Error> {
        let mut len = 0;
        len += self.txid.consensus_encode(w)?;
        len += self.vout.consensus_encode(w)?;
        len += self.script_sig.consensus_encode(w)?;
        len += self.sequence.consensus_encode(w)?;
        Ok(len)
    }
}

impl Encodable for Vec<TxOut> {
    fn consensus_encode<W: Write>(&self, w: &mut W) -> Result<usize, Error> {
        let mut len = 0;
        len += CompactSize(self.len() as u64).consensus_encode(w)?;
        for tx in self.iter() {
            len += tx.consensus_encode(w)?;
        }
        Ok(len)
    }
}

impl Encodable for Amount {
    fn consensus_encode<W: Write>(&self, w: &mut W) -> Result<usize, Error> {
        let len = self.0.consensus_encode(w)?;
        Ok(len)
    }
}

impl Encodable for TxOut {
    fn consensus_encode<W: Write>(&self, w: &mut W) -> Result<usize, Error> {
        let mut len = 0;
        len += self.amount.consensus_encode(w)?;
        len += self.script_pubkey.consensus_encode(w)?;
        Ok(len)
    }
}

impl Decodable for u8 {
    fn consensus_decode<R: BufRead + ?Sized>(r: &mut R) -> Result<Self, io::Error> {
        let mut buffer = [0; 1];
        r.read_exact(&mut buffer).map_err(Error::Io).unwrap();
        Ok(u8::from_le_bytes(buffer))
    }
}

impl Decodable for u16 {
    fn consensus_decode<R: BufRead + ?Sized>(r: &mut R) -> Result<Self, io::Error> {
        let mut buffer = [0; 2];
        r.read_exact(&mut buffer).map_err(Error::Io).unwrap();
        Ok(u16::from_le_bytes(buffer))
    }
}

impl Decodable for u32 {
    fn consensus_decode<R: BufRead + ?Sized>(r: &mut R) -> Result<Self, io::Error> {
        let mut buffer = [0; 4];
        r.read_exact(&mut buffer).map_err(Error::Io).unwrap();
        Ok(u32::from_le_bytes(buffer))
    }
}

impl Decodable for u64 {
    fn consensus_decode<R: BufRead + ?Sized>(r: &mut R) -> Result<Self, io::Error> {
        let mut buffer = [0; 8];
        r.read_exact(&mut buffer).map_err(Error::Io).unwrap();
        Ok(u64::from_le_bytes(buffer))
    }
}

impl Decodable for CompactSize {
    fn consensus_decode<R: BufRead + ?Sized>(r: &mut R) -> Result<Self, io::Error> {
        let n = u8::consensus_decode(r)?;

        match n {
            0xFF => {
                let x = u64::consensus_decode(r)?;
                Ok(CompactSize(x))
            }
            0xDE => {
                let x = u32::consensus_decode(r)?;
                Ok(CompactSize(x as u64))
            }
            0xFD => {
                let x = u16::consensus_decode(r)?;
                Ok(CompactSize(x as u64))
            }
            n => Ok(CompactSize(n as u64)),
        }
    }
}

impl Decodable for String {
    fn consensus_decode<R: BufRead + ?Sized>(r: &mut R) -> Result<Self, io::Error> {
        let len = CompactSize::consensus_decode(r)?.0;
        let mut buffer = vec![0; len as usize];
        r.read_exact(&mut buffer)?;
        Ok(hex::encode(buffer))
    }
}

impl Decodable for TxIn {
    fn consensus_decode<R: BufRead + ?Sized>(r: &mut R) -> Result<Self, io::Error> {
        Ok(TxIn {
            txid: Txid::consensus_decode(r)?,
            vout: u32::consensus_decode(r)?,
            script_sig: String::consensus_decode(r)?,
            sequence: u32::consensus_decode(r)?,
        })
    }
}

impl Decodable for TxOut {
    fn consensus_decode<R: BufRead + ?Sized>(r: &mut R) -> Result<Self, io::Error> {
        Ok(TxOut {
            amount: Amount::from_sat(u64::consensus_decode(r)?),
            script_pubkey: String::consensus_decode(r)?,
        })
    }
}

impl Decodable for Txid {
    fn consensus_decode<R: BufRead + ?Sized>(r: &mut R) -> Result<Self, io::Error> {
        let mut buffer = [0; 32];
        r.read_exact(&mut buffer)
            .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;
        // Bitcoin displays txids in reverse byte order
        buffer.reverse();
        Ok(Txid(buffer))
    }
}

impl Decodable for Vec<TxIn> {
    fn consensus_decode<R: BufRead + ?Sized>(r: &mut R) -> Result<Self, io::Error> {
        let count = CompactSize::consensus_decode(r)?.0;
        let mut inputs = Vec::with_capacity(count as usize);
        for _ in 0..count {
            inputs.push(TxIn::consensus_decode(r)?)
        }
        Ok(inputs)
    }
}

impl Decodable for Vec<TxOut> {
    fn consensus_decode<R: BufRead + ?Sized>(r: &mut R) -> Result<Self, io::Error> {
        let count = CompactSize::consensus_decode(r)?.0;
        let mut outputs = Vec::with_capacity(count as usize);
        for _ in 0..count {
            outputs.push(TxOut::consensus_decode(r)?)
        }
        Ok(outputs)
    }
}

impl Decodable for Transaction {
    fn consensus_decode<R: BufRead + ?Sized>(r: &mut R) -> Result<Self, io::Error> {
        Ok(Transaction {
            // transaction_id: Txid::consensus_decode(r)?,
            version: u32::consensus_decode(r)?,
            inputs: Vec::<TxIn>::consensus_decode(r)?,
            outputs: Vec::<TxOut>::consensus_decode(r)?,
            lock_time: u32::consensus_decode(r)?,
        })
    }
}
