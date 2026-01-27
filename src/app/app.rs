use eframe::egui;
use egui_toast::{Toast, ToastKind, ToastOptions, ToastStyle, Toasts};
use crate::{models::*, vault, krypt};

#[derive(Default)]
pub enum AppState {
    #[default]
    SelectVault,
    NewVault,
    OldVault,
    Unlocked,
}

#[derive(Default)]
pub struct PixelVaultApp {
  // UI state
  state: AppState,
  /// List of available vaults in filepaths
  available_vaults: Vec<String>,
  /// Actual vault selected out of available (filepath)
  selected_vault: Option<String>,

  /// Master password for the current vault
  pub(crate) master_password: String,
  /// Confirmation compared to master_password during vault creation
  pub(crate) master_password_confirm: String,

  // Vault creation
  pub(crate) new_vault_name: String,

  // Entry form fields
  pub(crate) new_service: String,
  pub(crate) new_username: String,
  pub(crate) new_password: String,

  /// Search for services / usernames
  pub(crate) search_query: String,

  // Data
  vault: Option<PasswordVault>,
  cipher_key: Option<Vec<u8>>,

  /// Index containing the password position to delete
  pub(crate) delete_confirmation_index: Option<usize>,

  // Display
  pub(crate) show_password_index: Option<usize>,
  decrypted_passwords: Vec<Option<String>>,
  // error_message: String,
  /// Replaces error_message, to show notifications and errors
  toasts: Toasts,
}

impl PixelVaultApp {
  pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
    // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
    // Restore app state using cc.storage (requires the "persistence" feature).
    // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
    // for e.g. egui::PaintCallback.
    let mut visuals = egui::Visuals::dark();
    visuals.window_corner_radius = 12.0.into();
    visuals.widgets.noninteractive.corner_radius = 8.0.into();
    cc.egui_ctx.set_visuals(visuals);

    std::fs::create_dir_all("vaults").ok();

    Self {
      state: AppState::SelectVault,
      available_vaults: vault::list_vaults(),
      toasts: Toasts::new()
        .anchor(egui::Align2::RIGHT_BOTTOM, (-10.0, -10.0))
        .direction(egui::Direction::BottomUp),
      ..Default::default()
    }
  }
    
  pub fn show_error(&mut self, message: impl Into<String>) {
    self.toasts.add(Toast {
      style: ToastStyle::default(),
      text: message.into().into(),
      kind: ToastKind::Error,
      options: ToastOptions::default()
        .duration_in_seconds(5.0)
        .show_progress(true)
        .show_icon(true),
    });
  }

  pub fn show_success(&mut self, message: impl Into<String>) {
    self.toasts.add(Toast {
      style: ToastStyle::default(),
      text: message.into().into(),
      kind: ToastKind::Success,
      options: ToastOptions::default()
        .duration_in_seconds(2.5)
        .show_progress(true)
        .show_icon(true),
    });
  }

  pub fn show_warning(&mut self, message: impl Into<String>) {
    self.toasts.add(Toast {
      style: ToastStyle::default(),
      text: message.into().into(),
      kind: ToastKind::Warning,
      options: ToastOptions::default()
        .duration_in_seconds(3.5)
        .show_progress(true)
        .show_icon(true),
    });
  }

  pub fn show_info(&mut self, message: impl Into<String>) {
    self.toasts.add(Toast {
      style: ToastStyle::default(),
      text: message.into().into(),
      kind: ToastKind::Info,
      options: ToastOptions::default()
        .duration_in_seconds(2.0)
        .show_progress(true)
        .show_icon(true),
    });
  }
  
  pub fn save_vault(&self) -> Result<(), String> {
    let vault = self.vault.as_ref().ok_or("No vault loaded")?;
    let path = self.selected_vault.as_ref().ok_or("No vault path")?;
    vault::save(path, vault)
  }
  
  /// Returns an immutable reference to the current vault if it exists
  pub fn get_current_vault(&self) -> Option<&PasswordVault> {
    self.vault.as_ref()
  }
  
  /// Decrypt password for a given index, returns error if vault or entries is invalid
  pub fn decrypted_password_for(&self, index: usize) -> Result<String, String> {
      let vault = self.vault.as_ref().ok_or("Vault not loaded")?;
      let entry = vault.entries.get(index).ok_or("Invalid index")?;
      let key = self.cipher_key.as_ref().ok_or("Invalid cipher key")?;
      krypt::decrypt_password(&key, &entry.encrypted_password, &entry.nonce)
  }
  
  pub fn attempt_create_vault(&mut self) {
    if self.new_vault_name.trim().is_empty() {
      self.show_error("Vault name cannot be empty");
    } else if self.master_password.is_empty() {
      self.show_error("Vault master password cannot be empty");
    } else if self.master_password != self.master_password_confirm {
      self.show_error("Passwords do not match");
    }
    // Fallback to empty if no new vault name (somehow ig)
    let _fallback_vault_name = chrono::Utc::now().timestamp().to_string();
    let path = format!("vaults/{}.json", 
      if !self.new_vault_name.is_empty() {self.new_vault_name.trim()} 
      else {&_fallback_vault_name}
    );
    if self.get_available_vaults().contains(&path) {
      self.show_error("Vault already exists");
    }
    self.state = AppState::Unlocked;
    self.selected_vault = Some(path.clone());
    match self.create_new_vault(&path).map_err(|e| e.to_string()) {
      Err(e) => self.show_error(&e),
      Ok(_) => self.show_success("Vault created successfully!")
    };
  }
  
  pub fn create_new_vault(&mut self, path: &str) -> Result<(), String> {
    let salt = krypt::gen_salt();
    let key = krypt::derive_key(&self.master_password, salt.as_str())?;
    let timestamp = chrono::Utc::now().timestamp().to_string();
    let (encrypted_timestamp, nonce) = krypt::encrypt_password(&key, &timestamp)
      .map_err(|e| e.to_string())?;
    self.vault = Some(PasswordVault {
      salt: salt.as_str().to_string(),
      entries: Vec::new(),
      verify: encrypted_timestamp,
      verify_nonce: nonce,
    });
    self.cipher_key = Some(key);
    self.selected_vault = Some(path.to_string());
    
    // Make sure the directory exists
    std::fs::create_dir_all("vaults").map_err(|e| e.to_string())?;
    
    // Save immediately
    self.save_vault()?;
    
    // Refresh available vaults
    self.available_vaults = vault::list_vaults();
    Ok(())
  }
  
  pub fn reload_available_vaults(&mut self) {
    self.available_vaults = vault::list_vaults();
  }
  
  pub fn select_existing_vault(&mut self, path: String) -> Result<(), String>{
    // vault::load(&path)?; // Same thing as below
    match vault::load(&path) {
      Ok(_) => {},
      Err(e) => {
        return Err(e)
      }
    }
    self.selected_vault = Some(path);
    self.state = AppState::OldVault;
    
    Ok(())
  }
  
  pub fn go_to_vault_creation(&mut self) {
    self.cipher_key = None;
    self.state = AppState::NewVault;
  }
  
  pub fn delete_vault(&mut self, path: &String) -> Result<(), String>{
    vault::delete(path)
  }
  
  pub fn lock_vault(&mut self) {
    self.state = AppState::SelectVault;
    self.master_password.clear();
    self.cipher_key = None;
    self.show_password_index = None;
    self.new_service.clear();
    self.new_username.clear();
    self.new_password.clear();
    self.vault = None;
    self.selected_vault = None;
    self.show_info("Vault locked");
  }
  
  /// Function to try to unlock the vault using self.master_password,
  /// returns false if fail and true if succeeded
  pub fn unlock(&mut self, path: &str) -> bool {
    // let path = match self.selected_vault.as_ref() {
    //   Some(p) => p,
    //   None => return false,
    // };
    let vault = match vault::load(&path) {
      Ok(v) => v,
      Err(_) => return false,
    };
    let salt = vault.salt.clone();
    self.vault = Some(vault);
    // let vault = self.vault.as_ref().unwrap();
    
    let key = match krypt::derive_key(&self.master_password, &salt) {
      Ok(k) => k,
      Err(_) => return false,
    };
    self.cipher_key = Some(key.clone());
    
    let vault = self.vault.as_ref().unwrap();
    
    krypt::verify(&key, &vault.verify, &vault.verify_nonce)
  }
  
  pub fn attempt_unlock(&mut self) -> Result<String, String> {
    if self.master_password.is_empty() {
      return Err("Master password cannot be empty!".into());
    }
    match self.state {
      AppState::NewVault => {
          // Make a new vault
          let path = self
            .selected_vault
            .clone()
            .unwrap_or_else(|| "vaults/new_vault.json".to_string());
          self.create_new_vault(&path)?;
          self.state = AppState::Unlocked;
          return Ok("Vault created successfully!".into());
      },
      AppState::OldVault => {
        let path = match self.get_selected_vault().clone() {
          Some(p) => p,
          None => {
            return Err("No vault selected!".into());
          }
        };
        if self.unlock(&path) {
          self.state = AppState::Unlocked;
          return Ok("Vault unlocked!".into());
        } else {
          self.master_password.clear();
          return Err("Incorrect Master Password".into());
        }
      },
      _ => {}
    }
    Ok("".into())
  }
  
  pub fn add_entry(&mut self) {
    let key: &[u8] = match self.cipher_key.as_deref() {
        Some(k) => k,
        None => {
            self.show_error("Vault is locked");
            return;
        }
    };
    match krypt::encrypt_password(key, &self.new_password) {
      Ok((encrypted, nonce)) => {
        let entry = PasswordEntry {
          service: self.new_service.clone(),
          username: self.new_username.clone(),
          encrypted_password: encrypted,
          nonce,
        };
        
        self.vault.as_mut().unwrap().entries.push(entry);

        match self.save_vault() {
          Ok(_) => {},
          Err(e) => {
            self.show_error(format!("Failed to save vault: {}", e));
          }
        };
        
        self.decrypted_passwords.push(None);

        self.new_service.clear();
        self.new_username.clear();
        self.new_password.clear();
      }
      Err(e) => {
        self.show_error(format!("Failed to encrypt: {}", e));
      }
    }
  }

  pub fn delete_entry(&mut self, index: usize) {
    if let Some(vault) = &mut self.vault {
      if index < vault.entries.len() {
        let service = vault.entries[index].service.clone();
        vault.entries.remove(index);
        

        self.show_info(format!("Deleted password for {}", service));

        // Delete password show if the entry is deleted
        if self.show_password_index == Some(index) {
          self.show_password_index = None;
        } else if let Some(pass_idx) = self.show_password_index {
          // Take one away if the index is after the deleted entry
          if pass_idx > index { 
            self.show_password_index = Some(pass_idx - 1);
          }
        }
      }
    }
  }
  
  pub fn num_entries(&self) -> Option<usize> {
    self.vault.as_ref().map(|v| v.entries.len())
  }
  
  pub fn get_available_vaults(&self) -> Vec<String> {
    self.available_vaults.clone()
  }
  
  pub fn get_selected_vault(&mut self) -> &Option<String> {
    &self.selected_vault
  }
  
  pub fn state(&self) -> &AppState {
      &self.state
  }
  
  pub fn back_to_vaults(&mut self) {
    self.state = AppState::SelectVault;
    self.master_password.clear();
    self.vault = None;
    self.cipher_key = None;
  }
}

impl eframe::App for PixelVaultApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.toasts.show(ctx);
        match self.state {
            AppState::SelectVault => self.show_select_vault(ctx),
            AppState::NewVault => self.show_new_vault(ctx),
            AppState::OldVault => self.show_old_vault(ctx),
            AppState::Unlocked => self.show_unlocked(ctx),
        }
    }
}
