use eframe::egui;
use crate::{app::PixelVaultApp};

impl PixelVaultApp{
  pub fn show_edit_entry(&mut self, ui: &mut egui::Ui) {
    ui.columns_const(|[col1, col2]| {
      col1.horizontal(|ui| {
        ui.heading("Edit Entry");
      });
      col2.horizontal(|ui| {
        self.change_feature_widget(ui);
      });
    });
    
    if self.vault.is_none() {
      ui.colored_label(egui::Color32::RED, "Vault doesn't exist!");
      return;
    }
    let edit_index = match self.edit_index {
      Some(i) => i,
      None => 0,
    };
    
    let vault = self.vault.as_mut().unwrap();
    // Check edit index out of bounds
    if edit_index >= vault.entries.len() {
      ui.colored_label(egui::Color32::RED, "Entry doesn't exist!");
      return;
    }
    
    let entry = &mut vault.entries[edit_index];
    

    ui.horizontal(|ui| {
      ui.label("Service:");
      ui.add(
        egui::TextEdit::singleline(&mut entry.username)
      )
      
    });

    ui.horizontal(|ui| {
      ui.label("Username:");
      ui.text_edit_singleline(&mut entry.service);
    });

    ui.horizontal(|ui| {
      ui.label("Password:");
      ui.add(egui::TextEdit::singleline(&mut entry.password));
    });

    ui.horizontal(|ui| {
      if ui.button("Save").clicked() {
        // handle save
      }
      if ui.button("Cancel").clicked() {
        // Handle cancel
      }
    });
  }
}
