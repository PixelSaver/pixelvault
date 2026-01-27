use serde::{Deserialize, Serialize};
// Data model used for passwords


#[derive(Serialize, Deserialize)]
pub struct PasswordVault {
  /// Public info used to derive master password
  pub salt: String,
  pub entries: Vec<PasswordEntry>,
  // Unencryption check for master password when entries is empty
  pub verify: Vec<u8>,
  pub verify_nonce: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PasswordEntry {
  pub service: String,
  pub username: String,
  pub encrypted_password: Vec<u8>,
  pub nonce: Vec<u8>,
}