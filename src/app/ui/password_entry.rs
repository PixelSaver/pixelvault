use eframe::egui;
use crate::app::PixelVaultApp;
use crate::models::{PasswordEntry};

impl PixelVaultApp {
  pub fn show_password_entry(&mut self, ui: &mut egui::Ui, entry: &PasswordEntry, index: usize) {
    PixelVaultApp::fancy_frame(ui).show(ui, |ui| {
      ui.set_width(ui.available_width());

      // Delete confirmation
      if self.delete_confirmation_index == Some(index) {
        ui.add_space(5.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
          ui.colored_label(egui::Color32::RED, "⚠ Delete this password?");
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
          ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Cancel").clicked() {
              self.delete_confirmation_index = None
            }
            if ui.button("Yes, delete").clicked() {
              self.delete_entry(index);
            }
          });
        });
        ui.add_space(5.0);
        return;
      }
      
      // Header row
      ui.columns_const(|[col1, col2]| {
        col1.horizontal(|ui| {
          ui.label(format!("🌐 {}", entry.service));
        });
        col2.horizontal(|ui| {
          ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            if ui.button("Delete").clicked() {
              self.delete_confirmation_index = Some(index);
            }
          });
        });
      });
      let response =
        ui.add(egui::Label::new(format!("👤 {}", entry.username)).sense(egui::Sense::click()));

      if response.clicked() {
        ui.ctx().copy_text(entry.username.clone());
        self.show_info("Username copied!");
      }

      response.on_hover_text("Click to copy username");

      // Password row
      ui.columns_const(|[col1, col2]| {
        col1.horizontal(|ui| {
          let password = {
            let vault = match self.vault.as_ref() {
              Some(v) => v,
              None => return,
            };
            &vault.entries[index].password
          };
          
          if Some(index) == self.show_password_index {
            let response =
              ui.add(egui::Label::new(format!("🔑 {}", password)).sense(egui::Sense::click()));
            if response.clicked() {
              ui.ctx().copy_text(password.clone());
              self.show_info("Password copied!")
            }
            response.on_hover_text("Click to copy");
          } else {
            // Password hidden, show dots and still do click to copy
            let response =
              ui.add(egui::Label::new("🔑 ••••••••••••••").sense(egui::Sense::click()));

            if response.clicked() {
              ui.ctx().copy_text(password.clone());
              self.show_info("Password copied!")
            }

            response.on_hover_text("Click to copy password");
          }
        });

        col2.horizontal(|ui| {
          ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            let (button_text, new_pass_idx) = if Some(index) == self.show_password_index {
              ("Hide", None)
            } else {
              ("Show", Some(index))
            };

            if ui.button(button_text).clicked() {
              self.show_password_index = new_pass_idx;
            }
          });
        });
      });
    });
    ui.add_space(5.0);
  }
} 