use eframe::egui;
use crate::app::{PixelVaultApp, app::{AppState, FeatureState}};

impl PixelVaultApp {
  pub fn change_feature_widget(&mut self, ui: &mut egui::Ui) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
      ui.menu_button("Change Mode", |ui| {
        if ui.button("Edit Entries").clicked() {
          if let AppState::Unlocked { feature_state } = &mut self.state_mut() {
            // Now you can modify feature_state
            *feature_state = FeatureState::EditEntry;
          }
          ui.close();
        }
        if ui.button("Add New Entries").clicked() {
          if let AppState::Unlocked { feature_state } = &mut self.state_mut() {
            *feature_state = FeatureState::NewEntry { show_pw_gen: false};
          }
          ui.close();
        }
      });
    });
  }
}
