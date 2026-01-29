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
    
    if let AppState::Unlocked { feature_state } = &mut self.state_mut() {
      let show_pw_gen = match feature_state {
        FeatureState::NewEntry { show_pw_gen } => show_pw_gen,
        _ => return,
      };
      if *show_pw_gen {
        self.show_password_generator(ui);
      }
    }
    
    ui.columns_const(|[col1, col2]| {
      col1.horizontal(|ui| {
        if ui.button("Add Entry").clicked() && !self.new_service.is_empty() {
          // Add entry here
          self.add_entry();
        }
      });
      col2.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
          if let AppState::Unlocked { feature_state } = &mut self.state_mut() {
            let show_pw_gen = match feature_state {
              FeatureState::NewEntry { show_pw_gen } => show_pw_gen,
              _ => return,
            };
            if *show_pw_gen {
              if ui.button("Hide Password Generator").clicked() {
                *feature_state = FeatureState::NewEntry { show_pw_gen: false }
              }
            } else {
              if ui.button("Show Password Generator").clicked() {
                *feature_state = FeatureState::NewEntry { show_pw_gen: true }
              }
            }
          }
        });
      })
    });
  }
}
