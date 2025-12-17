use std::net::SocketAddr;

use commonware_codec::ReadExt as _;
use commonware_cryptography::{
    Signer,
    ed25519::{PrivateKey, PublicKey},
};
use commonware_utils::from_hex_formatted;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Config {
    private_key: String,
    pub port: u16,
    pub peer_endpoint: String,
    pub peer_public_key: String,
}

impl Config {
    pub fn new(
        private_key: &PrivateKey,
        port: u16,
        peer_endpoint: &str,
        peer_public_key: &str,
    ) -> Self {
        Self {
            private_key: private_key.to_string(),
            port,
            peer_endpoint: peer_endpoint.into(),
            peer_public_key: peer_public_key.into(),
        }
    }

    /// Exports the configuration to the given filepath.
    /// The configuration is currently written to a YAML file.
    pub fn export(&self, filepath: &str) -> eyre::Result<()> {
        let path = std::path::Path::new(filepath);
        let dir_path = match path.parent() {
            Some(p) => p,
            None => return Err(eyre::eyre!("failed to get parent directory of config path")),
        };

        if !dir_path.exists() {
            std::fs::create_dir_all(dir_path)?;
        }

        std::fs::write(path, serde_yaml::to_string(self)?)?;

        Ok(())
    }

    /// Parses the configuration's private key.
    pub fn get_private_key(&self) -> PrivateKey {
        parse_private_key(&self.private_key).expect("invalid private key")
    }

    /// Parses the configuration's public key.
    pub fn get_public_key(&self) -> PublicKey {
        parse_private_key(&self.private_key)
            .expect("invalid private key")
            .public_key()
    }

    /// Retrieve a configuration stored in a given filepath.
    pub fn read(filepath: &str) -> eyre::Result<Self> {
        let contents = std::fs::read_to_string(std::path::Path::new(filepath))?;
        let config: Config = serde_yaml::from_str(&contents)?;

        Ok(config)
    }

    pub fn validate(&self) -> eyre::Result<()> {
        let _ = parse_private_key(&self.private_key)?;
        let _ = parse_public_key(&self.peer_public_key)?;
        let _ = parse_socket_addr(&self.peer_endpoint)?;

        Ok(())
    }
}

/// Builds the configuration file path for the given player ID.
pub fn get_config_path(public_key: &PublicKey) -> String {
    format!("./.battleship-commonware/config-{}.yaml", public_key)
}

/// Parses a hex-formatted ed25510 public key.
pub fn parse_public_key(input: &str) -> eyre::Result<PublicKey> {
    let public_key_bytes = from_hex_formatted(input).unwrap_or_default();
    let mut bz = public_key_bytes.as_slice();

    Ok(PublicKey::read(&mut bz)?)
}

/// Parses a hex-formatted ed25510 private key.
pub fn parse_private_key(input: &str) -> eyre::Result<PrivateKey> {
    let private_key_bytes = from_hex_formatted(input).unwrap_or_default();
    let mut bz = private_key_bytes.as_slice();

    Ok(PrivateKey::read(&mut bz)?)
}

/// Parses a socket address from the provided input.
pub fn parse_socket_addr(input: &str) -> eyre::Result<SocketAddr> {
    Ok(input.parse()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::NamedTempFile;

    #[test]
    fn test_new_config() {
        let config = Config::new(
            &PrivateKey::from_seed(0),
            5670,
            "127.0.0.1:5671".into(),
            "9a3744504560639ec670b7a17d492b273e077b0a96bef58ba7760779e544546e".into(),
        );

        assert!(!config.private_key.is_empty());
        assert_eq!(config.peer_endpoint, "127.0.0.1:5671");
        assert!(!config.peer_public_key.is_empty());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_config() {
        assert!(
            !Config::new(
                &PrivateKey::from_seed(0),
                5670,
                "abc",
                "9a3744504560639ec670b7a17d492b273e077b0a96bef58ba7760779e544546e",
            )
            .validate()
            .is_ok()
        );

        assert!(
            !Config::new(&PrivateKey::from_seed(0), 5671, "127.0.0.1:5670", "hij0123",)
                .validate()
                .is_ok()
        );
    }

    #[test]
    fn test_export_and_read() {
        let config = Config::new(
            &PrivateKey::from_seed(0),
            5670,
            "127.0.0.1:5671".into(),
            "9a3744504560639ec670b7a17d492b273e077b0a96bef58ba7760779e544546e".into(),
        );

        let filepath = NamedTempFile::new().expect("failed to get temporary filename");
        let path_string = &filepath.path().to_string_lossy();
        config.export(path_string).expect("failed to export");

        let read_config = Config::read(path_string).expect("failed to read config");
        assert_eq!(config, read_config);
    }
}
