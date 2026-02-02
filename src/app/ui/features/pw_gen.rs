use eframe::egui::{self, Color32, RichText, vec2};
use crate::app::app::PixelVaultApp;

impl PixelVaultApp {
  pub(crate) fn show_password_generator(&mut self, ui: &mut egui::Ui) {
    PixelVaultApp::fancy_frame(ui).outer_margin(0.0).show(ui, |ui| {
      ui.set_width(ui.available_width());
      ui.spacing_mut().item_spacing = vec2(0.0, 5.0);
      
      ui.label("Password Generator Here");
      egui::Frame::new()
        .corner_radius(0.0)
        .fill(ui.visuals().text_edit_bg_color.unwrap_or(Color32::from_gray(1)))
        .show(ui, |ui| {
          let gen_pass_response = ui.add(
            egui::Label::new(RichText::new(&self.pw_gen.generated_password)
              .color(
                ui.visuals().widgets.open.fg_stroke.color
              )
            )
          );
          if gen_pass_response.clicked() {
              ui.ctx().copy_text(self.pw_gen.generated_password.clone());
              self.show_info("Generated Password copied!");
          }
        });
          
      if ui.button("Generate Password").clicked() {
        self.pw_gen.generated_password = match self.pw_gen.generate() {
          Some(p) => p,
          None => return,
        };
      }
    });
  }
}