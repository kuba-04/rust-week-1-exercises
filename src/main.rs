use rust_week_1_exercises::{extract_tx_version, extract_tx_size};

fn main() {
    let transaction_hex = "0100000001a15d747f65d8b70680a844a21875f5a671d6070a6e9167e8d13a730b443768e1000000006b483045022100877123456789012345678901234567890123456789022079656090d7f6bac4c9a94e0aad311a4268e082a725f8aeae0573fb12ff866a5f01ffffffff0100f2052a010000001976a9148cb20a7664f2f69e5355aa427045bc15e7c6c77288ac00000000";

    let version = extract_tx_version(transaction_hex).unwrap();
    let input_count = extract_tx_size(transaction_hex);

    println!("Version: {:?}", version);
    println!("InputLength: {}", input_count);
}