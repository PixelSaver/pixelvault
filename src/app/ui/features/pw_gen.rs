use eframe::egui;
use crate::{app::app::PixelVaultApp};

impl PixelVaultApp {
  pub(crate) fn show_password_generator(&mut self, ui: &mut egui::Ui) {
    PixelVaultApp::fancy_frame(ui).outer_margin(0.0).show(ui, |ui| {
      ui.set_width(ui.available_width());
      
      ui.add(
        egui::Label::new(egui::RichText::new("Password Generator Here").heading().size(14.0))
      );
      ui.add_enabled(true,
        egui::TextEdit::singleline(&mut self.pw_gen.generated_password)
      );
      egui::Frame::default()
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .fill(ui.visuals().text_edit_bg_color())
        .inner_margin(3.0)
        .show(ui, |ui| {
          // ui.set_width(ui.available_width());
          ui.set_min_width((self.pw_gen.config.length as f32) * 8.0);
          ui.label(self.pw_gen.generated_password.clone());
        });
      if ui.button("Regenerate password").clicked() {
        self.pw_gen.generated_password = match self.pw_gen.generate() {
          Some(password) => password,
          None => {
            self.show_error("Failed to generate password");
            return;
          },
        };
      } 
    });
  }
}