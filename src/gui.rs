use crate::downloader;
use eframe::egui;

pub fn run_gui() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "crab-dl",
        options,
        Box::new(|_cc| Ok(Box::<CrabDL>::default())),
    )
}

#[derive(Default)]
struct CrabDL {
    current_url: String,
    urls: Vec<String>,
}

impl eframe::App for CrabDL {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let download_dir = "/home/benjamin/Downloads/";
        // for loading images
        egui_extras::install_image_loaders(ctx);
        // ui scaling
        ctx.set_pixels_per_point(1.2);

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.set_min_height(50.0);
                ui.add(egui::Image::new(egui::include_image!(
                    "../assets/rustacean-flat-happy.png"
                )));
                ui.heading(egui::RichText::new("crab-dl File Downloader").size(24.0));
            });
        });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .inner_margin(20)
                    .fill(ctx.style().visuals.window_fill()),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("URL:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.current_url)
                                .min_size(egui::vec2(280.0, 0.0)),
                        );
                    });
                    ui.add_space(5.0);
                    if ui.button("Add URL").clicked() && !self.current_url.is_empty() {
                        self.urls.push(self.current_url.clone());
                        self.current_url.clear();
                    }
                    ui.add_space(20.0);
                    ui.label("URL(s) to download:");
                    let mut to_remove = None;
                    for (idx, url) in self.urls.iter().enumerate() {
                        ui.vertical(|ui| {
                            ui.add(egui::Label::new(format!("{}. {}", idx + 1, url)).wrap());
                            if ui.button("Remove").clicked() {
                                to_remove = Some(idx);
                            }
                        });
                        ui.add_space(10.0);
                    }
                    if let Some(idx) = to_remove {
                        self.urls.remove(idx);
                    }
                    ui.separator();
                    ui.add_space(5.0);
                    if ui.button("Download all!").clicked() && !self.urls.is_empty() {
                        println!("Downloading {} files", self.urls.len());
                        let url_refs: Vec<&String> = self.urls.iter().collect();
                        downloader::initiate_download(download_dir, &url_refs);
                    }
                });
            });
    }
}
