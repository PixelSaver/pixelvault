use eframe::egui;
use crate::app::PixelVaultApp;
use egui_commonmark::{CommonMarkViewer, CommonMarkCache};

impl PixelVaultApp {
  /// UI for opening a new vault
  pub fn show_help(&mut self, ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
      ui.horizontal(|ui| {
        ui.heading("🔒 PixelVault");
      });
    });
    egui::CentralPanel::default().show(ctx, |ui| {
      PixelVaultApp::fancy_frame(ui).show(ui, |ui| {
        ui.set_width(ui.available_width());
        let markdown = PixelVaultApp::get_help_markdown();
        let mut cache = CommonMarkCache::default();
        ui.style_mut().url_in_tooltip = true;
        egui::ScrollArea::vertical().show(ui, |ui| {
          CommonMarkViewer::new().show(ui, &mut cache, &markdown);
        });
      });
    });
  }
}