//! The `vault` module handles vault persistence.
//!
//! Responsibilities:
//! - Load a vault from a file.
//! - Save a vault to a file.
//! - Delete a vault.
//! - List all available vaults
use crate::models::EncryptedVault;
use std::fs;

/// Reads the README and places it into a string
const README: &str = include_str!("../README.md");

/// Load the encrypted vault from disk using filesystem
/// 
/// # Arguments
/// - `path`: path of the wanted file
/// 
/// # Returns
/// An [`EncryptedVault`] with the salt, nonce, and encrypted vault data.
/// 
/// # Errors
/// Returns a string error if the file does not exist or is not a valid JSON file.
pub fn load(path: &str) -> Result<EncryptedVault, String> {
  let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
  let vault = serde_json::from_str(&data).map_err(|e| e.to_string())?;
  Ok(vault)
}

/// Save a given encrypted vault at a path
/// 
/// # Arguments
/// - `path`: the path at which the data will be saved
/// - `vault`: `EncryptedVault` to be serialized and saved
/// 
/// # Errors
/// Returns an error if the file cannot be written to or cannot be JSON serialized
pub fn save(path: &str, vault: &EncryptedVault) -> Result<(), String> {
  let json = serde_json::to_string_pretty(vault).map_err(|e| e.to_string())?;
  fs::write(path, json).map_err(|e| e.to_string())
}

/// Gets all vaults in the `vaults/` directory, and returns default if failed
/// 
/// # Returns
/// - `vault_list`: A vec containing all relative vault filepaths
/// 
/// # Errors
/// If the read_dir operation fails, it returns an empty vec.
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

/// Deletes the vault at a given filepath.
/// 
/// # Errors
/// Returns an error if the filesystem remove fails
pub fn delete(path: &String) -> Result<(), String> {
  fs::remove_file(path).map_err(|e| e.to_string())
}

/// Returns the `README.md` as a string
pub fn get_readme() -> String {
  README.to_string()
}
