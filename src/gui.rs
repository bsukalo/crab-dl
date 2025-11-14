use crate::downloader;
use eframe::egui;
use rfd::FileDialog;
use std::f32::INFINITY;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, channel};

pub fn run_gui() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "crab-dl",
        options,
        Box::new(|_cc| Ok(Box::<CrabDL>::default())),
    )
}

struct CrabDL {
    current_url: String,
    download_dir: String,
    urls: Vec<String>,
    dir_receiver: Option<Receiver<PathBuf>>,
    error: Option<String>,
}

impl Default for CrabDL {
    fn default() -> Self {
        let download_dir = dirs::home_dir()
            .and_then(|p| p.to_str().map(|s| format!("{}/Downloads/", s)))
            .unwrap_or_else(|| "./".to_string());
        Self {
            current_url: String::new(),
            download_dir,
            urls: Vec::new(),
            dir_receiver: None,
            error: None,
        }
    }
}

impl eframe::App for CrabDL {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let download_dir = format!("{}", &self.download_dir);
        // for loading images
        egui_extras::install_image_loaders(ctx);
        // ui scaling
        ctx.set_pixels_per_point(1.2);

        if let Some(rx) = &self.dir_receiver {
            if let Ok(path) = rx.try_recv() {
                self.download_dir = path.to_string_lossy().to_string();
                self.dir_receiver = None;
            }
        }

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
                        ui.label("Download directory:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.download_dir)
                                .desired_width(INFINITY),
                        );
                    });
                    ui.add_space(5.0);
                    if ui.button("Choose Download Directory").clicked() {
                        let (tx, rx) = channel();
                        self.dir_receiver = Some(rx);
                        let ctx = ctx.clone();

                        std::thread::spawn(move || {
                            if let Some(path) = FileDialog::new().pick_folder() {
                                tx.send(path).ok();
                                ctx.request_repaint(); // Wake up the UI
                            }
                        });
                    }
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        ui.label("URL:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.current_url)
                                .desired_width(INFINITY),
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
                        println!("Downloading {} file(s)", self.urls.len());
                        let url_refs: Vec<&String> = self.urls.iter().collect();

                        match downloader::initiate_download(&download_dir, &url_refs) {
                            Ok(_) => {
                                self.error = None;
                                println!("Downloads completed!");
                            }
                            Err(e) => {
                                self.error = Some(format!("Download failed: {}", e));
                            }
                        }
                    }
                    if let Some(error_msg) = &self.error {
                        ui.colored_label(egui::Color32::RED, error_msg);
                        ui.add_space(5.0);
                    }
                });
            });
    }
}
