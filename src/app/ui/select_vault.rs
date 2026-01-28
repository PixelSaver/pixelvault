use crate::app::PixelVaultApp;
use eframe::egui;

impl PixelVaultApp {
  pub fn show_select_vault(&mut self, ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
      ui.columns_const(|[col1, col2]| {
        col1.horizontal(|ui| {
          ui.heading("🔒 PixelVault");
        });
        col2.horizontal(|ui| {
          ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            ui.horizontal(|ui| {
              if ui.button("Help").clicked() {
                self.go_to_help();
              };
            });
          });
        });
      });
    });
    egui::CentralPanel::default().show(ctx, |ui| {
      PixelVaultApp::fancy_frame(ui).show(ui, |ui| {
        ui.heading("Choose a Vault");
        ui.add_space(10.0);
        
        let available_vaults = self.get_available_vaults();

        if available_vaults.is_empty() {
          ui.label("No vaults found. Create one to get started!");
          ui.add_space(10.0);
        } else {
          for vault_ref in available_vaults {
            let vault_path = vault_ref.clone();
            let display_name = vault_path
              .trim_start_matches("vaults\\")
              .trim_start_matches("vaults/")
              .trim_end_matches(".json");

            let mut select_clicked = false;
            let mut delete_clicked = false;

            ui.columns_const(|[col1, col2]| {
              col1.horizontal(|ui| {
                if ui.button(display_name).clicked() {
                  select_clicked = true;
                }
              });
              col2.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                  if ui.small_button("🗑").on_hover_text("Delete vault").clicked() {
                    delete_clicked = true;
                  }
                });
              });
            });
            if select_clicked {
              match self.select_existing_vault(vault_path.clone()) {
                Ok(_) => self.show_info(format!("Vault selected: {}", display_name)),
                Err(e) => self.show_error(format!("Vault failed to load: {}", e)),
              }
            }
            if delete_clicked {
              if let Err(e) = self.delete_vault(&vault_path) {
                self.show_error(format!("Failed to delete vault: {}", e));
              } else {
                self.show_info(format!("Deleted vault '{}'", display_name));
                self.reload_available_vaults();
              }
            }
          }
        }

        ui.separator();

        if ui.button("➕ Create New Vault").clicked() {
          // let vault_name = format!("vault_{}", chrono::Utc::now().timestamp());
          // let path = format!("vaults/{}.json", vault_name);
          // self.attempt_create_vault(&path);
          self.go_to_vault_creation();
        }
      });
    });
  }
}
