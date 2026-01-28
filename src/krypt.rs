use aes_gcm::{
  Aes256Gcm, Nonce,
  aead::{Aead, AeadCore, KeyInit, OsRng, rand_core::RngCore},
};
use argon2::Argon2;

use crate::models::{EncryptedVault, PasswordVault};
// Everything to do with cryptography

pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32], String> {
  let argon2 = Argon2::default();
  let mut key = [0u8; 32];

  argon2
    .hash_password_into(password.as_bytes(), salt, &mut key)
    .map_err(|e| format!("Key derivation failed: {}", e.to_string()))?;

  Ok(key)
}

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

pub fn decrypt_password(key: &[u8], encrypted: &[u8], nonce: &[u8]) -> Result<String, String> {
  let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;
  let nonce = Nonce::from_slice(nonce);

  let plaintext = cipher
    .decrypt(nonce, encrypted)
    .map_err(|_| "Decryption failed")?;

  String::from_utf8(plaintext).map_err(|e| e.to_string())
}

pub fn gen_salt() -> [u8; 16] {
  let mut salt = [0u8; 16];
  OsRng.fill_bytes(&mut salt);
  salt
}

pub fn verify(key: &[u8], encrypted: &[u8], nonce: &[u8]) -> bool {
  let cipher = match Aes256Gcm::new_from_slice(&key) {
    Ok(c) => c,
    Err(_) => return false,
  };
  let nonce = Nonce::from_slice(&nonce);
  if cipher.decrypt(nonce, encrypted.as_ref()).is_err() {
    return false;
  }
  true
}
