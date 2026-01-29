use eframe::egui;
use crate::app::PixelVaultApp;

impl PixelVaultApp{
  pub fn show_edit_entry(&mut self, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
      ui.horizontal(|ui| {
        ui.heading("Edit Entry");
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
    });
  }
}
