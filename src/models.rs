//! The `models` module handles data models used for the vault.
//! 
//! # Responsibilities
//! - Defines data structures for storing and managing passwords.
//! - Implements serialization and deserialization for data persistence.
use serde::{Deserialize, Serialize};

/// Plaintext vault stored only in local memory
#[derive(Serialize, Deserialize)]
pub struct PasswordVault {
  /// Decrypted entries for passwords
  pub entries: Vec<PasswordEntry>,
}

/// Encrypted on-disk representation of the vault.
/// 
/// Contains all the information (except master password) needed
/// to decrypt the vault.
#[derive(Serialize, Deserialize)]
pub struct EncryptedVault {
  /// Random Argon2 salt
  /// Must be unique per vault.
  pub salt: [u8; 16],
  /// AES-GCM nonce used for vault encryption
  /// Must never be reused with the same key.
  pub nonce: [u8; 12],
  /// Entire encrypted vault (serialized)
  pub ciphertext: Vec<u8>,
}

/// Plaintext password entry containing a service, username, and password.
#[derive(Serialize, Deserialize, Clone)]
pub struct PasswordEntry {
  pub service: String,
  pub username: String,
  pub password: String,
}
