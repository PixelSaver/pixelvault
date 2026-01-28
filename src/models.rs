use serde::{Deserialize, Serialize};
// Data model used for passwords

/// Plaintext vault stored only in local memory
#[derive(Serialize, Deserialize)]
pub struct PasswordVault {
  /// Decrypted entries for passwords
  pub entries: Vec<PasswordEntry>,
}

#[derive(Serialize, Deserialize)]
pub struct EncryptedVault {
  /// Argon2 salt
  pub salt: [u8; 16],
  /// AES-GCM nonce
  pub nonce: [u8; 12],
  /// Entire encrypted vault
  pub ciphertext: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PasswordEntry {
  pub service: String,
  pub username: String,
  pub password: String,
}
