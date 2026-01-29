use eframe::egui;
use crate::app::app::PixelVaultApp;

impl PixelVaultApp {
  pub(crate) fn show_password_generator(&mut self, ui: &mut egui::Ui) {
    ui.label("Password Generator Here");
  }
}