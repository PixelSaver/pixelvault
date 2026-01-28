use eframe::egui;
use crate::app::PixelVaultApp;

impl PixelVaultApp {
  /// Static helper to provide the same frame across the application
  pub fn fancy_frame(ui: &egui::Ui) -> egui::Frame {
    egui::Frame::new()
      .inner_margin(12)
      .outer_margin(6)
      .corner_radius(14)
      .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
      .fill(ui.visuals().panel_fill)
    // .shadow(ui.visuals().popup_shadow)
  }
}