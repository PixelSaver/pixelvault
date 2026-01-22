use aes_gcm::{
Aes256Gcm, Nonce,
aead::{Aead, AeadCore, KeyInit, OsRng},
};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use chrono;
use eframe::egui;
use egui::{Align, InnerResponse, Layout, Ui};
use serde::{Deserialize, Serialize};
use std::fs;

fn main() {
  let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 600.0]),
    ..Default::default()
  };
  eframe::run_native(
    "PixelVault",
    options,
    Box::new(|cc| Ok(Box::new(PixelVaultApp::new(cc)))),
  )
  .unwrap(); // DEBUG Panics.
}

#[derive(Serialize, Deserialize)]
struct PasswordVault {
  /// Public info used to derive master password
  salt: String,
  entries: Vec<PasswordEntry>,
  // Unencryption check for master password when entries is empty
  // check: String,
  // TODO
}

#[derive(Serialize, Deserialize, Clone)]
struct PasswordEntry {
  service: String,
  username: String,
  encrypted_password: Vec<u8>,
  nonce: Vec<u8>,
}

enum AppState {
  SelectVault,
  Locked { is_new: bool },
  Unlocked,
}
impl Default for AppState {
  fn default() -> Self {
    AppState::SelectVault
  }
}

#[derive(Default)]
struct PixelVaultApp {
  // UI state
  state: AppState,
  /// List of available vaults in filepaths
  available_vaults: Vec<String>,
  /// Actual vault selected out of available
  selected_vault: Option<String>,

  master_password: String,

  // Entry form fields
  new_service: String,
  new_username: String,
  new_password: String,

  // Data
  vault: Option<PasswordVault>,
  cipher_key: Option<Vec<u8>>,

  // Display
  show_password_index: Option<usize>,
  decrypted_passwords: Vec<Option<String>>,
  error_message: String,
}

impl PixelVaultApp {
  fn load_available_vaults() -> Vec<String> {
    let mut vaults = Vec::new();

    if let Ok(entries) = fs::read_dir("vaults") {
      for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
          vaults.push(path.to_string_lossy().to_string());
        }
      }
    }

    vaults
  }

  fn fancy_frame(&self, ui: &egui::Ui) -> egui::Frame {
    egui::Frame::new()
      .inner_margin(12)
      .outer_margin(6)
      .corner_radius(14)
      .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
      .fill(ui.visuals().panel_fill)
    // .shadow(ui.visuals().popup_shadow)
  }

  fn show_locked(&mut self, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
      self.fancy_frame(ui).show(ui, |ui| {
        ui.set_width(ui.available_width());

        if let Some(vault_name) = &self.selected_vault {
          let display_name = vault_name
            .trim_start_matches("vaults/")
            .trim_end_matches(".json");
          ui.label(format!("Vault: {}", display_name));
        }

        // Login screen
        match &self.state {
          AppState::Locked { is_new } => {
            if *is_new {
              ui.label("Set the master password:");
            } else {
              ui.label("Enter the correct master password");
            }
          }
          _ => {}
        }
        let response = ui.add(
          egui::TextEdit::singleline(&mut self.master_password)
            .password(true)
            .hint_text("Master password"),
        );

        // If lost focus or enter key
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
          self.attempt_unlock();
        }

        if ui.button("Unlock").clicked() {
          self.attempt_unlock();
        }

        if !self.error_message.is_empty() {
          ui.add_space(10.0);
          ui.colored_label(egui::Color32::RED, &self.error_message);
        }

        ui.add_space(10.0);

        if ui.button("Back to Vaults").clicked() {
          self.state = AppState::SelectVault;
          self.master_password.clear();
          self.error_message.clear();
          self.vault = None;
          self.cipher_key = None;
        }
      });
    });
  }

  fn show_unlocked(&mut self, ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
      ui.horizontal(|ui| {
        ui.heading("🔓 PixelVault");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
          if ui.button("🔒 Lock").clicked() {
            self.lock_vault();
          }
        });
      });
    });
    egui::CentralPanel::default().show(ctx, |ui| {
      self.fancy_frame(ui).show(ui, |ui| {
        ui.set_width(ui.available_width());

        // Main interface
        ui.horizontal(|ui| {
          ui.heading("Add New Password");
        });

        ui.horizontal(|ui| {
          ui.label("Service:");
          ui.text_edit_singleline(&mut self.new_service);
        });

        ui.horizontal(|ui| {
          ui.label("Username:");
          ui.text_edit_singleline(&mut self.new_username);
        });

        ui.horizontal(|ui| {
          ui.label("Password:");
          ui.add(egui::TextEdit::singleline(&mut self.new_password).password(true));
        });

        if ui.button("Add Entry").clicked() && !self.new_service.is_empty() {
          // Add entry here
          self.add_entry();
        }

        ui.separator();
        ui.heading("Stored Passwords");

        // Clone entries to avoid borrow checker issues
        if let Some(vault) = &mut self.vault {
          if vault.entries.is_empty() {
            ui.label("No stored passwords yet.");
          } else {
            let entries = vault.entries.clone();

            egui::ScrollArea::vertical()
              .auto_shrink(false)
              .show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                  for (i, entry) in entries.iter().enumerate() {
                    self.fancy_frame(ui).outer_margin(3).show(ui, |ui| {
                      ui.set_width(ui.available_width());
                      ui.columns_const(|[col1, col2]| {
                        col1.horizontal(|ui| {
                          ui.label(format!("🌐 {}", entry.service));
                        });
                        col2.horizontal(|ui| {
                          ui.with_layout(
                            egui::Layout::right_to_left(
                              egui::Align::Min,
                            ),
                            |ui| {
                              if ui.button("Delete").clicked() {
                                self.delete_entry(i);
                              }
                            },
                          );
                        });
                      });
                      ui.label(format!("👤 {}", entry.username));

                      ui.columns_const(|[col1, col2]| {
                        if Some(i) == self.show_password_index {
                          col1.horizontal(|ui| {
                            match self.decrypt_password(
                              &entry.encrypted_password,
                              &entry.nonce,) {
                                Ok(password) => {
                                  ui.label(format!("🔑 {}", password));
                                }
                                Err(e) => {
                                  ui.colored_label(
                                    egui::Color32::RED,
                                    format!("Error: {}", e),
                                  );
                                }
                              }});
                          col2.horizontal(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                              if ui.button("Hide").clicked() {
                                // Hide the password
                                self.show_password_index = None;
                              }
                            });
                          });
                        } else {
                          col1.horizontal(|ui| {
                            ui.label("🔑 ••••••••••••••••••")
                          });
                          col2.horizontal(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                              if ui.button("Show Password").clicked() {
                                // Reveal the password
                                self.show_password_index = Some(i);
                              }
                            });
                          });
                          
                        }
                      });
                    });
                    ui.add_space(5.0);
                  }
                });
              });
          }
        }
      });
    });
  }

  fn show_select_vault(&mut self, ui: &mut egui::Ui) {
    self.fancy_frame(ui).show(ui, |ui| {
      ui.heading("Choose a Vault");
      ui.add_space(10.0);

      for vault in self.available_vaults.clone() {
        ui.columns(2, |columns| {
          columns[0].horizontal(|ui| {
            if ui.button(&vault).clicked() {
              self.selected_vault = Some(vault.clone());
              self.state = AppState::Locked { is_new: false };
              self.load_vault_from_path(&vault).unwrap();
            }
          });
          columns[1].horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
              if ui.button("Remove vault").clicked() {
                if let Some(pos) =
                self.available_vaults.iter().position(|x| *x == vault)
                {
                  self.available_vaults.swap_remove(pos);
                }
              }
            });
          });
        });
      }

      ui.separator();

      if ui.button("➕ Create New Vault").clicked() {
        let vault_name = format!("vault_{}", chrono::Utc::now().timestamp());
        let path = format!("vaults/{}.json", vault_name);
        self.selected_vault = Some(path.clone());
        self.state = AppState::Locked { is_new: true };
      }

      if !self.error_message.is_empty() {
        ui.add_space(10.0);
        ui.colored_label(egui::Color32::RED, &self.error_message);
      }
    });
  }

  fn load_vault_from_path(&mut self, path: &str) -> Result<(), String> {
    let data = fs::read_to_string(path).map_err(|e| format!("Failed to read vault: {}", e))?;
    let vault: PasswordVault =
      serde_json::from_str(&data).map_err(|e| format!("Invalid vault format: {}", e))?;
    self.vault = Some(vault);
    self.selected_vault = Some(path.to_string());
    Ok(())
  }

  /// Function to try to unlock the vault using self.master_password,
  /// returns false if fail and true if succeeded
  fn unlock(&mut self) -> bool {
    let vault = match self.vault.as_ref() {
      Some(v) => v,
      None => return false,
    };
    // let vault = self.vault.as_ref().unwrap();
    let key = match self.derive_key(&self.master_password, &vault.salt) {
      Ok(k) => k,
      Err(_) => return false,
    };
    // let key = self.derive_key(&self.master_password, &vault.salt);

    // Verify the password, try encrypting the first entry
    if let Some(entry) = vault.entries.first() {
      let cipher = match Aes256Gcm::new_from_slice(&key) {
        Ok(c) => c,
        Err(_) => return false,
      };
      let nonce = Nonce::from_slice(&entry.nonce);
      if cipher
        .decrypt(nonce, entry.encrypted_password.as_ref())
        .is_err()
      {
        return false;
      }
    }
    // self.cipher_key = Some(key);

    self.cipher_key = Some(key);
    true
    // self.state = AppState::Unlocked;
  }

  fn attempt_unlock(&mut self) {
    if self.master_password.is_empty() {
      self.error_message = "Master password cannot be empty".to_string();
      return;
    }
    match self.state {
      AppState::Locked { is_new } => {
        if is_new {
          // Make a new vault
          let path = self
            .selected_vault
            .clone()
            .unwrap_or_else(|| "vaults/new_vault.json".to_string());
          self.create_new_vault(&path);
          self.state = AppState::Unlocked;
          self.error_message.clear();
        } else {
          if self.unlock() {
            self.state = AppState::Unlocked;
            self.error_message.clear();
          } else {
            self.error_message = "Incorrect master password".to_string();
            self.master_password.clear();
          }
        }
      }
      _ => {}
    }
    // let vault = self.vault.as_ref().unwrap();
    // let key = self.derive_key(&self.master_password, &vault.salt);
    // self.cipher_key = Some(key);
    // self.state = AppState::Unlocked;
  }

  fn lock_vault(&mut self) {
    self.state = AppState::SelectVault;
    self.master_password.clear();
    self.cipher_key = None;
    self.show_password_index = None;
    self.new_service.clear();
    self.new_username.clear();
    self.new_password.clear();
    self.vault = None;
    self.selected_vault = None;
    self.error_message.clear();
  }

  fn create_new_vault(&mut self, path: &str) {
    let salt = SaltString::generate(&mut OsRng);
    let key = self
      .derive_key(&self.master_password, salt.as_str())
      .expect("Failed to derive key");
    self.vault = Some(PasswordVault {
      salt: salt.as_str().to_string(),
      entries: Vec::new(),
    });
    self.cipher_key = Some(key);
    self.selected_vault = Some(path.to_string());

    // Make sure the directory exists
    std::fs::create_dir_all("vaults").ok();

    // Save immediately
    self.save_vault();

    // Refresh available vaults
    self.available_vaults = Self::load_available_vaults();
  }

  fn derive_key(&self, password: &str, salt: &str) -> Result<Vec<u8>, String> {
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

  fn encrypt_password(&self, password: &str) -> Result<(Vec<u8>, Vec<u8>), String> {
    let key = self.cipher_key.as_ref().ok_or("Not unlocked")?;
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;

    // Use OsRng for cryptographic randomness
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher
      .encrypt(&nonce, password.as_bytes())
      .map_err(|e| e.to_string())?;

    Ok((ciphertext, nonce.to_vec()))
  }

  fn decrypt_password(&self, encrypted: &[u8], nonce: &[u8]) -> Result<String, String> {
    let key = self.cipher_key.as_ref().ok_or("Not unlocked")?;
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(nonce);

    let plaintext = cipher
      .decrypt(nonce, encrypted)
      .map_err(|_| "Decryption failed")?;

    String::from_utf8(plaintext).map_err(|e| e.to_string())
  }

  /// Save the vault using json to passwords.json
  fn save_vault(&self) {
    if let Some(vault) = &self.vault {
      if let Some(path) = &self.selected_vault {
        let json = serde_json::to_string_pretty(vault).unwrap();
        if let Err(e) = fs::write(path, json) {
          eprintln!("Failed to save vault: {}", e);
        }
      }
    }
  }

  fn add_entry(&mut self) {
    if self.new_service.is_empty() {
      return;
    }

    match self.encrypt_password(&self.new_password) {
      Ok((encrypted, nonce)) => {
        let entry = PasswordEntry {
          service: self.new_service.clone(),
          username: self.new_username.clone(),
          encrypted_password: encrypted,
          nonce,
        };

        if let Some(vault) = &mut self.vault {
          vault.entries.push(entry);
        }
        self.decrypted_passwords.push(None);

        self.save_vault();

        self.new_service.clear();
        self.new_username.clear();
        self.new_password.clear();
      }
      Err(e) => {
        self.error_message = format!("Failed to encrypt: {}", e);
      }
    }
  }

  fn delete_entry(&mut self, index: usize) {
    if let Some(vault) = &mut self.vault {
      if index < vault.entries.len() {
        vault.entries.remove(index);
        self.save_vault();

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

  fn new(cc: &eframe::CreationContext<'_>) -> Self {
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
      available_vaults: Self::load_available_vaults(),
      ..Default::default()
    }
  }
}

impl eframe::App for PixelVaultApp {
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
match &self.state {
AppState::SelectVault => {
egui::CentralPanel::default().show(ctx, |ui| {
self.show_select_vault(ui);
});
}
AppState::Locked { .. } => {
self.show_locked(ctx);
}
AppState::Unlocked => {
self.show_unlocked(ctx);
}
}
}
}
