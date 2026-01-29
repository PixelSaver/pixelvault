use eframe::egui;
use crate::app::{PixelVaultApp, app::FeatureState};

impl PixelVaultApp {  
  /// UI depicting an unlocked vault.
  pub fn show_unlocked(&mut self, ctx: &egui::Context, feature_state: &FeatureState) {
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
      ui.horizontal(|ui| {
        ui.heading("🔓 PixelVault");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
          if ui.button("🔒 Lock").clicked() {
            self.lock_vault();
          }
        });
      });
    });
    egui::CentralPanel::default().show(ctx, |ui| {
      PixelVaultApp::fancy_frame(ui).show(ui, |ui| {
        ui.set_width(ui.available_width());

        match feature_state {
        FeatureState::EditEntry { show_pw_gen }
        }

        ui.separator();
        ui.horizontal(|ui| {
          ui.heading("Stored Passwords");
          if let Some(num_entries) = self.num_entries() {
            ui.label(format!("({})", num_entries));
          }
        });
        // Service / Entry search query
        ui.horizontal(|ui| {
          ui.label("Search:");
          let response =  ui.add(
            egui::TextEdit::singleline(&mut self.search_query)
              .hint_text("Search services or usernames")
              .desired_width(ui.available_width()-30.0),
          );
          response.on_hover_text("Try searching something!");
          let clear_response = ui.small_button("X");
          if clear_response.clicked() {
            self.search_query.clear();
          }
          clear_response.on_hover_text("Clear search query")
        });

        ui.separator();

        if let Some(vault) = self.get_current_vault() {
          let mut results = vault.search_entries(&self.search_query);
          results.sort_by(|a, b| b.2.cmp(&a.2)); // sort by score
          
          // Make results only have references to Password Entry
          let results: Vec<_> = results.into_iter()
            .map(|(i, entry, score)| (i, entry.clone(), score))
            .collect();
          
          if results.is_empty() {
            ui.vertical_centered(|ui| {
              ui.add_space(20.0);
              if self.search_query.is_empty() {
                ui.label("No stored passwords yet.");
                ui.label("Add your first password to get started!");
              } else {
                ui.label("No matching passwords found.");
              }
            });
          } else {
            egui::ScrollArea::vertical()
              .auto_shrink(false)
              .show(ui, |ui| {
                for (i, entry, _score) in results {
                  self.show_password_entry(ui, &entry, i);
                }
              });
          }
        }
      });
    });
  }
}