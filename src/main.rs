use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use eframe::egui;
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
    /// Public info used to derive master password (not implemented)
    salt: String,
    entries: Vec<PasswordEntry>,
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
    Locked { vault_path: String },
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
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            self.fancy_frame(ui).show(ui, |ui| {
                ui.set_width(ui.available_width());

                // Login screen
                ui.label("Enter master password:");
                let _response = ui.add(
                    egui::TextEdit::singleline(&mut self.master_password)
                        .password(true)
                        .hint_text("Master password"),
                );

                if ui.button("Unlock").clicked() {
                    self.unlock();
                }
            });
        });
    }

    fn show_unlocked(&mut self, ctx: &egui::Context) {
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
                    let entries = vault.entries.clone();

                    egui::ScrollArea::vertical()
                        .auto_shrink(false)
                        .show(ui, |ui| {
                            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                for (i, entry) in entries.iter().enumerate() {
                                    self.fancy_frame(ui).outer_margin(3).show(ui, |ui| {
                                        ui.set_width(ui.available_width());
                                        ui.label(format!("🌐 {}", entry.service));
                                        ui.label(format!("👤 {}", entry.username));

                                        ui.horizontal(|ui| {
                                            if Some(i) == self.show_password_index {
                                                if let Ok(password) = self.decrypt_password(
                                                    &entry.encrypted_password,
                                                    &entry.nonce,
                                                ) {
                                                    // self.decrypted_passwords[i] = Some(password);
                                                    ui.label(format!("🔑 {}", password));
                                                }
                                                if ui.button("Hide").clicked() {
                                                    // Hide the password
                                                    self.show_password_index = None;
                                                }
                                            } else {
                                                if ui.button("Show Password").clicked() {
                                                    // Reveal the password
                                                    self.show_password_index = Some(i);
                                                }
                                            }
                                        });
                                    });
                                    ui.add_space(5.0);
                                }
                            });
                        });
                }
            });
        });
    }

    fn show_select_vault(&mut self, ui: &mut egui::Ui) {
        self.fancy_frame(ui).show(ui, |ui| {
            ui.heading("Choose a Vault");
            ui.add_space(10.0);

            for vault in self.available_vaults.clone() {
                if ui.button(&vault).clicked() {
                    self.selected_vault = Some(vault.clone());
                    self.state = AppState::Locked {
                        vault_path: vault.clone(),
                    };
                    self.load_vault_from_path(&vault).unwrap();
                }
            }

            ui.separator();

            if ui.button("➕ Create New Vault").clicked() {
                self.create_new_vault("vaults/new_vault.json");
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


    fn unlock(&mut self) {
        let vault = self.vault.as_ref().unwrap();
        let key = self.derive_key(&self.master_password, &vault.salt);
        self.cipher_key = Some(key);
        self.state = AppState::Unlocked;
    }

    fn create_new_vault(&mut self, path: &str) {
        let salt = SaltString::generate(&mut OsRng);
        let key = self.derive_key(&self.master_password, salt.as_str());
        self.vault = Some(PasswordVault {
            salt: salt.as_str().to_string(),
            entries: Vec::new(),
        });
        self.cipher_key = Some(key);
        self.state = AppState::Unlocked;
        self.selected_vault = Some(path.to_string());
    
        // Make sure the directory exists
        std::fs::create_dir_all("vaults").ok();
    
        // Save immediately
        self.save_vault();
    
        // Refresh available vaults
        self.available_vaults = Self::load_available_vaults();
    }


    fn derive_key(&self, password: &str, salt: &str) -> Vec<u8> {
        let salt = SaltString::from_b64(salt).unwrap();
        let argon2 = Argon2::default();
        let hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();
        hash.hash.unwrap().as_bytes()[..32].to_vec()
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

    /// Tries to load vault from passwords.json
    fn load_vault(&mut self) {
        if let Ok(data) = fs::read_to_string("passwords.json") {
            self.vault = serde_json::from_str(&data).ok();
        }
    }

    fn add_entry(&mut self) {
        if let Ok((encrypted, nonce)) = self.encrypt_password(&self.new_password) {
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
        egui::CentralPanel::default().show(ctx, |ui| {
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
        });
    }
}
