use clap::{arg, Parser};

#[derive(Parser)]
#[command(name = "Transaction decoder")]
#[command(version = "1.0")]
#[command(about = "Bitcoin tx decored", long_about = None)]
pub struct Cli {
    #[arg(required = true, help = "(string, required) raw transaction hex")]
    pub transaction_hex: String,
}
