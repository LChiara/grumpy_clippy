use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use eframe::{
    App, CreationContext, Frame,
    egui::{self, FontId},
    epaint::PathStroke,
};
use egui::{Color32, TextureHandle};

use crate::app_state::SharedAppState;

pub struct ClippyApp {
    clippy_texture: Option<TextureHandle>,
    state: SharedAppState,
    running: Arc<AtomicBool>,
}

impl ClippyApp {
    pub fn new(cc: &CreationContext<'_>, state: SharedAppState, running: Arc<AtomicBool>) -> Self {
        let clippy_texture = load_clippy_texture(cc);
        Self {
            clippy_texture,
            state,
            running,
        }
    }

    fn draw_speech_bubble(&self, ui: &mut egui::Ui) {
        let rect = ui.max_rect().intersect(ui.available_rect_before_wrap());
        let bubble_position = egui::Pos2::new(rect.center().x - 230.0, rect.top() - 250.0);
        let bubble_size = egui::vec2(270.0, 80.0);
        let bubble_rect = egui::Rect::from_min_size(bubble_position, bubble_size);

        ui.painter().rect_filled(
            bubble_rect,
            egui::Rounding::same(15),
            Color32::from_black_alpha(120),
        );

        let triangle_points = vec![
            bubble_position + egui::vec2(270.0, 48.0),
            bubble_position + egui::vec2(295.7, 63.0),
            bubble_position + egui::vec2(270.0, 60.0),
        ];

        let polygon = egui::Shape::convex_polygon(
            triangle_points,
            Color32::from_black_alpha(120),
            PathStroke::default(),
        );
        ui.painter().add(polygon);

        let message = self.state.read().unwrap().message.clone();
        ui.painter().text(
            bubble_position + egui::vec2(10.0, 10.0),
            egui::Align2::LEFT_TOP,
            message,                                           // split_message(&message),
            FontId::new(11.0, egui::FontFamily::Proportional), // Size in points
            Color32::WHITE,
        );
    }
}

impl App for ClippyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        if !self.running.load(Ordering::SeqCst) {
            std::process::exit(0); // Hard exit for now 
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Push the image to the far right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(tex) = &self.clippy_texture {
                        ui.image(tex);
                    } else {
                        ui.label("[Missing Clippy Image]");
                    }
                });

                // Optional: leave space or put a heading on the left
                ui.vertical(|ui| {
                    ui.heading("Grumpy Clippy");
                });
            });

            self.draw_speech_bubble(ui);
        });
    }
}

fn load_clippy_texture(cc: &CreationContext<'_>) -> Option<TextureHandle> {
    const CLIPPY_BYTES: &[u8] = include_bytes!("../assets/clippy.png");
    image::load_from_memory(CLIPPY_BYTES).ok().map(|image| {
        let resized = image
            .resize_exact(150, 150, image::imageops::FilterType::Lanczos3)
            .to_rgba8();
        let size = [resized.width() as usize, resized.height() as usize];
        let pixels = resized.into_vec();
        cc.egui_ctx.load_texture(
            "clippy",
            egui::ColorImage::from_rgba_unmultiplied(size, &pixels),
            Default::default(),
        )
    })
}

fn split_message(input: &str) -> String {
    let splitted: Vec<String> = input
        .chars()
        .collect::<Vec<char>>()
        .chunks(40)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect();
    splitted.join("\n")
}
