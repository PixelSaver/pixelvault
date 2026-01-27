use aes_gcm::{
  Aes256Gcm, Nonce,
  aead::{Aead, AeadCore, KeyInit, OsRng},
};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
// Everything to do with cryptography

pub fn derive_key(password: &str, salt: &str) -> Result<Vec<u8>, String> {
  let salt = SaltString::from_b64(salt).map_err(|e| format!("Invalid salt: {}", e))?;
  let argon2 = Argon2::default();
  let hash = argon2
    .hash_password(password.as_bytes(), &salt)
    .map_err(|e| format!("Key derivation failed: {}", e))?;
  Ok(hash.hash.unwrap().as_bytes()[..32].to_vec())
  // let salt = SaltString::from_b64(salt).unwrap();
  // let argon2 = Argon2::default();
  // let hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();
  // hash.hash.unwrap().as_bytes()[..32].to_vec()
}

pub fn encrypt_password(key: &[u8], password: &str) -> Result<(Vec<u8>, Vec<u8>), String> {
  let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;

  // Use OsRng for cryptographic randomness
  let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

  let ciphertext = cipher
    .encrypt(&nonce, password.as_bytes())
    .map_err(|e| e.to_string())?;

  Ok((ciphertext, nonce.to_vec()))
}

pub fn decrypt_password(key: &[u8], encrypted: &[u8], nonce: &[u8]) -> Result<String, String> {
  let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;
  let nonce = Nonce::from_slice(nonce);

  let plaintext = cipher
    .decrypt(nonce, encrypted)
    .map_err(|_| "Decryption failed")?;

  String::from_utf8(plaintext).map_err(|e| e.to_string())
}

pub fn gen_salt() -> String {
  SaltString::generate(&mut OsRng).to_string()
}