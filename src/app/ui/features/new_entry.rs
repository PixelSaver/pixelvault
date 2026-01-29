use eframe::egui;
use crate::app::{PixelVaultApp, app::FeatureState, app::AppState};

impl PixelVaultApp{
  /// UI depicting a form to add a new entry (username, service, password)
  pub fn show_new_entry(&mut self, ui: &mut egui::Ui) {
    
    ui.columns_const(|[col1, col2]| {
      col1.horizontal(|ui| {
        ui.heading("New Entry");
      });
      col2.horizontal(|ui| {
        self.change_feature_widget(ui);
      });
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
  }
}
