use eframe::egui;
use crate::app::PixelVaultApp;

impl PixelVaultApp{
  /// UI depicting a form to add a new entry (username, service, password)
  pub fn show_new_entry(&mut self, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
      ui.horizontal(|ui| {
        ui.heading("New Entry");
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
        ui.add(egui::TextEdit::singleline(&mut self.new_password));
      });
      
      if ui.button("Add Entry").clicked() && !self.new_service.is_empty() {
        // Add entry here
        self.add_entry();
      }
    });
  }
}
