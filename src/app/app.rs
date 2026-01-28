use crate::{krypt, models::*, vault};
use eframe::egui;
use egui_toast::{Toast, ToastKind, ToastOptions, ToastStyle, Toasts};

/// State of the application, from selection, opening vaults, to help screen
#[derive(Default)]
pub enum AppState {
  #[default]
  SelectVault,
  NewVault,
  OldVault,
  Unlocked,
  Help,
}

/// App state variables
#[derive(Default)]
pub struct PixelVaultApp {
  /// UI state
  state: AppState,
  /// List of available vaults in filepaths
  available_vaults: Vec<String>,
  /// Actual vault selected out of available (filepath)
  selected_vault: Option<String>,

  /// Master password for the current vault
  pub(crate) master_password: String,
  /// Confirmation compared to master_password during vault creation
  pub(crate) master_password_confirm: String,

  /// Vault name used when making a new vault
  pub(crate) new_vault_name: String,

  // Entry form fields
  pub(crate) new_service: String,
  pub(crate) new_username: String,
  pub(crate) new_password: String,

  /// Search query for services / usernames when `AppState::Unlocked`
  pub(crate) search_query: String,

  // Data
  /// Plaintext vault stored only in local memory
  pub(crate) vault: Option<PasswordVault>,
  /// Encrypted vault stored in file
  pub(crate) encrypted_vault: Option<EncryptedVault>,

  /// Index containing the password position to delete
  pub(crate) delete_confirmation_index: Option<usize>,

  // Display
  /// Index storing shown password index so only one is shown at a time
  pub(crate) show_password_index: Option<usize>,
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
  
  /// Uses `egui_toast` to show error notification
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
  
  /// Uses `egui_toast` to show success notification
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

  /// Uses `egui_toast` to show warning notification
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

  /// Uses `egui_toast` to show info notification
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

  /// Encrypts and saves current vault state.
  /// 
  /// # Errors
  /// Returns an error if:
  /// - Vault / vault path doesn't exist
  /// - Encryption fails
  /// - Filesystem save fails
  pub fn save_vault(&mut self) -> Result<(), String> {
    let plaintext = self.vault.as_ref().ok_or("Vault locked")?;
    let path = self.selected_vault.as_ref().ok_or("No vault path")?;

    let encrypted = krypt::encrypt_vault(plaintext, &self.master_password)?;

    vault::save(path, &encrypted)?;
    self.encrypted_vault = Some(encrypted);
    Ok(())
  }

  /// Returns an immutable reference to the current vault if it exists
  pub fn get_current_vault(&self) -> Option<&PasswordVault> {
    self.vault.as_ref()
  }

  /// Try to create a vault
  /// 
  /// # Errors
  /// Shows user an error if
  /// - Vault name / master password is empty
  /// - Master password confirm is different
  /// - A vault at the path already exists
  /// - Initializing a vault fails
  pub fn attempt_create_vault(&mut self) {
    if self.new_vault_name.trim().is_empty() {
      self.show_error("Vault name cannot be empty");
      return;
    } else if self.master_password.is_empty() {
      self.show_error("Vault master password cannot be empty");
      return;
    } else if self.master_password != self.master_password_confirm {
      self.show_error("Passwords do not match");
      return;
    }
    // Fallback to empty if no new vault name (somehow ig)
    let _fallback_vault_name = chrono::Utc::now().timestamp().to_string();
    let path = format!(
      "vaults/{}.json",
      if !self.new_vault_name.is_empty() {
        self.new_vault_name.trim()
      } else {
        &_fallback_vault_name
      }
    );
    if self.get_available_vaults().contains(&path) {
      self.show_error("Vault already exists");
      return;
    }
    self.state = AppState::Unlocked;
    self.selected_vault = Some(path.clone());
    match self.create_new_vault(&path).map_err(|e| e.to_string()) {
      Err(e) => self.show_error(&e),
      Ok(_) => self.show_success("Vault created successfully!"),
    };
  }
  
  /// Creating a new vault
  /// 
  /// # Errors
  /// Shows user an error if
  /// - Vault encryption fails
  /// - Filesystem save fails
  pub fn create_new_vault(&mut self, path: &str) -> Result<(), String> {
    let plaintext = PasswordVault { entries: vec![] };

    let encrypted = krypt::encrypt_vault(&plaintext, &self.master_password)?;

    vault::save(path, &encrypted)?;

    self.vault = Some(plaintext);
    self.encrypted_vault = Some(encrypted);
    Ok(())
  }

  /// Reads filesystem available vaults
  pub fn reload_available_vaults(&mut self) {
    self.available_vaults = vault::list_vaults();
  }

  /// Select an existing vault
  /// 
  /// # Errors
  /// Returns an error if:
  /// - Vault fails to load
  pub fn select_existing_vault(&mut self, path: String) -> Result<(), String> {
    // vault::load(&path)?; // Same thing as below
    let encrypted = vault::load(&path)?;
    self.encrypted_vault = Some(encrypted);
    self.selected_vault = Some(path.to_string());
    self.state = AppState::OldVault;

    Ok(())
  }

  /// Change to `AppState::NewVault`
  pub fn go_to_vault_creation(&mut self) {
    self.state = AppState::NewVault;
  }
  
  /// Change to `AppState::Help`
  pub fn go_to_help(&mut self) {
    self.state = AppState::Help;
  }

  /// Delete a vault
  /// 
  /// # Errors
  /// Returns an error if the filesystem delete fails
  pub fn delete_vault(&mut self, path: &String) -> Result<(), String> {
    vault::delete(path)
  }

  /// Reset PixelVaultApp struct data and exits the vault.
  pub fn lock_vault(&mut self) {
    self.state = AppState::SelectVault;
    self.master_password.clear();
    self.master_password_confirm.clear();
    self.show_password_index = None;
    self.new_service.clear();
    self.new_username.clear();
    self.new_password.clear();
    self.vault = None;
    self.selected_vault = None;
    self.show_info("Vault locked");
  }

  /// Function to try to unlock the vault using self.master_password,
  /// 
  /// # Returns
  /// False and true based on success
  pub fn unlock(&mut self, path: &str) -> bool {
    let encrypted = match vault::load(&path) {
      Ok(v) => v,
      Err(_) => return false,
    };

    let plaintext = match krypt::decrypt_vault(&encrypted, &self.master_password) {
      Ok(v) => v,
      Err(_) => return false,
    };

    self.encrypted_vault = Some(encrypted);
    self.vault = Some(plaintext);
    true
  }

  /// Attempt to unlock the vault stored in the PixelVaultApp state data
  /// 
  /// # Returns
  /// Success string
  /// 
  /// # Errors
  /// Returns an error if:
  /// - Master password is empty
  /// - `self.create_new_vault()` fails
  /// - Master password is incorrect
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
      }
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
      }
      _ => {}
    }
    Ok("".into())
  }

  /// Add a `PasswordEntry` to the vault entries based on PixelVaultApp state data
  /// 
  /// # Errors
  /// Shows the user an error if the vault is still locked or saving fails
  pub fn add_entry(&mut self) {
    let vault = match self.vault.as_mut() {
      Some(v) => v,
      None => {
        self.show_error("Vault is locked");
        return;
      }
    };

    vault.entries.push(PasswordEntry {
      service: self.new_service.clone(),
      username: self.new_username.clone(),
      password: self.new_password.clone(),
    });

    self.new_service.clear();
    self.new_username.clear();
    self.new_password.clear();

    self.save_vault().unwrap_or_else(|e| {
      self.show_error(e.to_string());
    })
  }

  /// Delete an entry at a given index. Decrements show_password_index for bounds safety
  /// 
  /// # Returns
  /// Shows user an info popup if password is removed
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

  /// Returns number of password entries in the current vault.
  pub fn num_entries(&self) -> Option<usize> {
    self.vault.as_ref().map(|v| v.entries.len())
  }

  /// Returns a copy of the available vaults
  pub fn get_available_vaults(&self) -> Vec<String> {
    self.available_vaults.clone()
  }

  /// Return a reference to a selected vault
  pub fn get_selected_vault(&mut self) -> &Option<String> {
    &self.selected_vault
  }

  pub fn state(&self) -> &AppState {
    &self.state
  }

  /// From `AppState::NewVault` or `AppState::OldVault` to `AppState::SelectVault`
  /// Clears the input boxes
  pub fn back_to_vaults(&mut self) {
    self.state = AppState::SelectVault;
    self.new_vault_name.clear();
    self.master_password.clear();
    self.master_password_confirm.clear();
    self.vault = None;
  }
  
  /// Return the readme as a string
  pub fn get_help_markdown() -> String {
    vault::get_readme()
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
      AppState::Help => self.show_help(ctx),
      // _ => {
      //   self.lock_vault();
      //   self.state = AppState::SelectVault;
      //   self.show_select_vault(ctx);
      // }
    }
  }
}
