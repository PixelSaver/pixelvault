//! The `krypt` module handles cryptography.
//!
//! Responsibilities:
//! - Derive the key from the password and a random salt.
//! - Decrypt / Encrypt the entire vault using AES-GCM.
//! 
//! # Security model
//! - Master password is never stored (if lost, the vault is locked forever)
//! - Encryption keys are derived from the master password using Argon2.
//! - Vault data is encrypted as a single item.
//! - Nonces are regenerated randomly each encryption
use aes_gcm::{
  Aes256Gcm, 
  aead::{Aead, AeadCore, KeyInit, OsRng, rand_core::RngCore},
};
use argon2::Argon2;

use crate::models::{EncryptedVault, PasswordVault};

/// Derives a 256-bit encryption key from a master password and a salt.
/// 
/// # Arguments 
/// - `password`: The master password used for key derivation.
/// - `salt`: A random salt used for key derivation.
/// 
/// # Returns
/// A 32-bit key suitable for AES-256-GCM
/// 
/// # Errors
/// Returns an error if key derivation fails.
/// 
/// # Security
/// - Argon2id with default parameters.
/// - Returned key must not be persisted.
pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32], String> {
  let argon2 = Argon2::default();
  let mut key = [0u8; 32];

  argon2
    .hash_password_into(password.as_bytes(), salt, &mut key)
    .map_err(|e| format!("Key derivation failed: {}", e.to_string()))?;

  Ok(key)
}

/// Encrypts plaintext vault using a master password.
/// 
/// Serializes the vault, derives a key, and encrypts the serialized data.
/// 
/// # Security
/// - Encrypts data with AES-256-GCM.
/// - Nonce is generated using OsRng each time.
/// 
/// # Errors 
/// Returns an error if key derivation, encryption, or serialization fails.
pub fn encrypt_vault(
  vault: &PasswordVault,
  master_password: &str,
) -> Result<EncryptedVault, String> {
  let salt: [u8; 16] = gen_salt();
  let key = derive_key(master_password, &salt)?;
  let cipher = Aes256Gcm::new_from_slice(&key)
    .map_err(|e| e.to_string())?;

  // Use OsRng for cryptographic randomness
  let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
  
  // Serialize from vault => bytes
  let serialized = serde_json::to_vec(&vault).map_err(|e| e.to_string())?;

  let ciphertext = cipher
    .encrypt(&nonce, serialized.as_ref())
    .map_err(|e| format!("Encryption failed: {}", e.to_string()))?;

  Ok(EncryptedVault {
    salt,
    nonce: nonce.into(),
    ciphertext,
  })
}

/// Decrypts plaintext vault using a master password.
/// 
/// Serializes the vault, derives a key, and decrypts the serialized data.
/// 
/// # Security
/// - Decrypts data with AES-256-GCM.
/// - Nonce is generated using OsRng each time.
/// 
/// # Errors 
/// Returns an error if key derivation, encryption, or serialization fails.
pub fn decrypt_vault(
  encrypted: &EncryptedVault,
  master_password: &str,
) -> Result<PasswordVault, String> {
  let key = derive_key(master_password, &encrypted.salt)?;
  let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;

  let plaintext = cipher
    .decrypt(&encrypted.nonce.into(), encrypted.ciphertext.as_ref())
    .map_err(|e| format!("Invalid password: {}", e))?;

  // Deserialize from bytes => vault
  let vault = serde_json::from_slice(&plaintext)
    .map_err(|e| e.to_string())?;
  
  Ok(vault)
}

/// Generates a random salt for key derivation.
/// 
/// # Security
/// - Uses OsRng for cryptographic randomness.
/// 
/// # Errors 
/// Returns an error if OsRng fails to generate random bytes.
pub fn gen_salt() -> [u8; 16] {
  let mut salt = [0u8; 16];
  OsRng.fill_bytes(&mut salt);
  salt
}