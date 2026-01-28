use crate::app::PixelVaultApp;
use eframe::egui;

impl PixelVaultApp {
  /// UI for opening a new vault
  pub fn show_old_vault(&mut self, ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
      ui.horizontal(|ui| {
        ui.heading("🔒 PixelVault");
      });
    });
    egui::CentralPanel::default().show(ctx, |ui| {
      PixelVaultApp::fancy_frame(ui).show(ui, |ui| {
        ui.set_width(ui.available_width());

        if let Some(vault_name) = &self.get_selected_vault() {
          let display_name = vault_name
            .trim_start_matches("vaults\\")
            .trim_start_matches("vaults/")
            .trim_end_matches(".json");
          ui.label(format!("Vault: {}", display_name));
          ui.add_space(10.0);
        }
        ui.label("Enter the correct master password");
        let response = ui.add(
          egui::TextEdit::singleline(&mut self.master_password)
            .password(true)
            .desired_width(ui.available_width())
            .hint_text("Master password"),
        );
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
          match self.attempt_unlock() {
            Ok(m) => {
              if !m.is_empty() {
                self.show_success(m)
              }
            }
            Err(e) => {
              self.show_error(e);
            }
          };
        }

        if ui.button("Unlock").clicked() {
          match self.attempt_unlock() {
            Ok(m) => {
              if !m.is_empty() {
                self.show_success(m)
              }
            }
            Err(e) => {
              self.show_error(e);
            }
          };
        }

        ui.add_space(10.0);

        if ui.button("Back to Vaults").clicked() {
          self.back_to_vaults();
        }
      });
    });
  }
}
