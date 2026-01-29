//! PixelVault -- A secure offline password manager.
//! 
//! PixelVault is a standalone GUI application for managing encrypted credentials locally.
//! It uses modern cryptography and doesn't rely on external or cloud services.
//! Your data is yours. PixelVault does its best to keep it that way.
//! 
//! # Overview 
//! - egui-based desktop UI
//! - Argon2 key derivation
//! - AES-GCM authenticated encryption
//! 
//! This crate is intended to be run as an executable, not used as a library. 
mod app;  
mod krypt;
mod models;
mod vault;
mod search;
mod pw_gen;

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