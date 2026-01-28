use eframe::egui;
use crate::app::PixelVaultApp;

impl PixelVaultApp {
  /// UI for opening a new vault
  pub fn show_new_vault(&mut self, ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
      ui.horizontal(|ui| {
        ui.heading("🔒 PixelVault");
      });
    });
    egui::CentralPanel::default().show(ctx, |ui| {
      PixelVaultApp::fancy_frame(ui).show(ui, |ui| {
        ui.set_width(ui.available_width());

        ui.label("Create a new vault");
        let name_response = ui.add(
          egui::TextEdit::singleline(&mut self.new_vault_name)
            .desired_width(ui.available_width())
            .hint_text("Vault Name"),
        );
        if name_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
          self.attempt_create_vault();
        }
        // Grab focus on first enter
        if self.new_vault_name.is_empty() && !name_response.has_focus() {
          name_response.request_focus();
        }

        let pass_response = ui.add(
          egui::TextEdit::singleline(&mut self.master_password)
            .password(true)
            .desired_width(ui.available_width())
            .hint_text("Choose a strong master password"),
        );
        if pass_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
          self.attempt_create_vault();
        }
        let pass_confirm_response = ui.add(
          egui::TextEdit::singleline(&mut self.master_password_confirm)
            .password(true)
            .desired_width(ui.available_width())
            .hint_text("Confirm your master password"),
        );
        if pass_confirm_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
        {
          self.attempt_create_vault();
        }

        ui.add_space(10.0);

        ui.columns_const(|[col1, col2]| {
          col1.horizontal(|ui| {
            if ui.button("Back to Vaults").clicked() {
              self.back_to_vaults();
            }
          });
          col2.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
              if ui.button("Create New Vault").clicked() {
                self.attempt_create_vault();
              }
            });
          });
        });
      });
    });
  }
}