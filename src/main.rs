mod app;  
mod krypt;
mod models;
mod vault;
mod search;

use eframe::egui;
use app::PixelVaultApp;

fn main() {
  let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 600.0]),
    ..Default::default()
  };
  eframe::run_native(
    "PixelVault",
    options,
    Box::new(|cc| Ok(Box::new(PixelVaultApp::new(cc)))),
  )
  .unwrap(); // DEBUG Panics.
}