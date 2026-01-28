//! The `vault` module handles vault persistence.
//!
//! Responsibilities:
//! - Load a vault from a file.
//! - Save a vault to a file.
//! - Delete a vault.
//! - List all available vaults
use crate::models::EncryptedVault;
use std::fs;

pub fn load(path: &str) -> Result<EncryptedVault, String> {
  let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
  let vault = serde_json::from_str(&data).map_err(|e| e.to_string())?;
  Ok(vault)
}

pub fn save(path: &str, vault: &EncryptedVault) -> Result<(), String> {
  let json = serde_json::to_string_pretty(vault).map_err(|e| e.to_string())?;
  fs::write(path, json).map_err(|e| e.to_string())
}

/// Gets all vaults in the `vaults/` directory, and returns default if failed
pub fn list_vaults() -> Vec<String> {
  fs::read_dir("vaults")
    .map(|entries| {
      entries
        .flatten()
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("json"))
        .map(|e| e.path().to_string_lossy().to_string())
        .collect()
    })
    .unwrap_or_default()
}

pub fn delete(path: &String) -> Result<(), String> {
  fs::remove_file(path).map_err(|e| e.to_string())
}
