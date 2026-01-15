use eframe::egui;
use serde::{Deserialize, Serialize};

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
    encrypted_password: Vec<u8>,
    nonce: Vec<u8>,
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
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

impl eframe::App for PixelVaultApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.is_unlocked {
                ui.heading("Unlock Vault");
                ui.add(egui::TextEdit::singleline(&mut self.master_password).password(true));

                if ui.button("Unlock").clicked() {
                    // placeholder logic
                    self.is_unlocked = true;
                }
            } else {
                ui.heading("Vault");
            }
        });
    }
}
