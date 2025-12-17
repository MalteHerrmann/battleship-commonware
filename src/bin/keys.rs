use clap::arg;
use commonware_cryptography::{PrivateKeyExt as _, Signer, ed25519::PrivateKey};

fn main() {
    let command = clap::Command::new("battleship-commonware-public-key")
        .arg(arg!(--id <ID> "the player ID"));

    let args = command.get_matches();
    let private_key = PrivateKey::from_seed(
        args.get_one::<String>("id")
            .expect("must set --id")
            .parse::<u64>()
            .expect("invalid id")
    );

    println!("private key: {}", private_key);
    println!("public key: {}", private_key.public_key());
}
