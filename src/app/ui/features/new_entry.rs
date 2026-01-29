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
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
          ui.menu_button("Change Mode", |ui| {
            if ui.button("Edit Entries").clicked() {
              if let AppState::Unlocked { feature_state } = &mut self.state_mut() {
                // Now you can modify feature_state
                *feature_state = FeatureState::EditEntry { show_pw_gen: false };
              }
              ui.close();
            }
            if ui.button("Add New Entries").clicked() {
              if let AppState::Unlocked { feature_state } = &mut self.state_mut() {
                *feature_state = FeatureState::NewEntry;
              }
              ui.close();
            }
          });
        });
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
