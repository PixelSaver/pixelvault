use eframe::egui;
use serde::{Deserialize, Serialize};
use std::fs;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "PixelVault",
        native_options,
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
    // encrypted_password: Vec<u8>,
    encrypted_password: String,
    // nonce: Vec<u8>,
}

#[derive(Default)]
struct PixelVaultApp {
    // UI state
    master_password: String,
    is_unlocked: bool,

    // Entry form fields
    new_service: String,
    new_username: String,
    new_password: String,

    // Data
    vault: Option<PasswordVault>,
    cipher_key: Option<Vec<u8>>,
    entries: Vec<PasswordEntry>,
}

impl PixelVaultApp {
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
                        .hint_text("Master password")
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
                   ui.add(egui::TextEdit::singleline(&mut self.new_password)
                       .password(true)
                   );
               });
               
               if ui.button("Add Entry").clicked() && !self.new_service.is_empty() {
                   // Add entry here
                   self.add_entry();
               }
            });
        });
    }
    
    fn unlock(&mut self) {
        if let Some(_vault) = &self.vault {
            self.is_unlocked = true;
        } else {
            // Create new vault
            self.vault = Some(PasswordVault {
                salt: "todo".into(),
                entries: vec![],
            });
            self.is_unlocked = true;
        }
    }
    
    /// Save the vault using json to passwords.json
    fn save_vault(&self) {
        if let Some(vault) = &self.vault {
            let json = serde_json::to_string_pretty(vault).unwrap();
            fs::write("passwords.json", json).ok();
        }
    }
    
    /// Tries to load vault from passwords.json
    fn load_vault(&mut self) {
        if let Ok(data) = fs::read_to_string("passwords.json") {
            self.vault = serde_json::from_str(&data).ok();
        }
    }
    
    fn add_entry(&mut self) {
        let entry = PasswordEntry {
            service: self.new_service.clone(),
            username: self.new_username.clone(),
            encrypted_password: self.new_password.clone(),
        };
        
        self.entries.push(entry);
        
        self.new_service.clear();
        self.new_username.clear();
        self.new_password.clear();
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

        Self::default()
    }
}

impl eframe::App for PixelVaultApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("PixelSaver Password Manager");
            ui.add_space(10.0);
            
            if !self.is_unlocked {
                self.show_locked(ctx)
            } else {
                // Unlocked
                self.show_unlocked(ctx);
            }
        });
    }
}
