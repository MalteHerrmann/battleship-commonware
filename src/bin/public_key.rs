use battleship_commonware::config::config::parse_private_key;
use clap::arg;
use commonware_cryptography::Signer;

fn main() {
    let command = clap::Command::new("battleship-commonware-public-key")
        .args([arg!(--"private-key" <PRIV_KEY> "the private key for which to get the public key")]);

    let args = command.get_matches();
    let private_key = parse_private_key(
        args.get_one::<String>("private-key")
            .expect("must set --private-key"),
    )
    .expect("failed to decode hex key");

    println!("public key: {}", private_key.public_key());
}
