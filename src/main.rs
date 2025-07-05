// src/main.rs
use eframe::egui;
use rfd::FileDialog;
use std::fs;
use std::path::PathBuf;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RpadApp {
    content: String,
    current_file: Option<PathBuf>,
    is_modified: bool,
    font_size: f32,
    word_wrap: bool,
    show_about: bool,
    find_text: String,
    replace_text: String,
    show_find_replace: bool,
    status_bar: bool,
}

impl Default for RpadApp {
    fn default() -> Self {
        Self {
            content: String::new(),
            current_file: None,
            is_modified: false,
            font_size: 14.0,
            word_wrap: true,
            show_about: false,
            find_text: String::new(),
            replace_text: String::new(),
            show_find_replace: false,
            status_bar: true,
        }
    }
}

impl RpadApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            if let Some(app_str) = storage.get_string(eframe::APP_KEY) {
                if let Ok(app) = serde_json::from_str::<RpadApp>(&app_str) {
                    return app;
                }
            }
        }
        Default::default()
    }

    fn new_file(&mut self) {
        if self.is_modified {
            // In a real app, you'd show a save dialog here
        }
        self.content.clear();
        self.current_file = None;
        self.is_modified = false;
    }

    fn open_file(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("Text Files", &["txt"])
            .add_filter("All Files", &["*"])
            .pick_file()
        {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    self.content = content;
                    self.current_file = Some(path);
                    self.is_modified = false;
                }
                Err(e) => {
                    eprintln!("Failed to open file: {}", e);
                }
            }
        }
    }

    fn save_file(&mut self) {
        if let Some(path) = &self.current_file {
            self.save_to_path(path.clone());
        } else {
            self.save_as_file();
        }
    }

    fn save_as_file(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("Text Files", &["txt"])
            .save_file()
        {
            self.save_to_path(path);
        }
    }

    fn save_to_path(&mut self, path: PathBuf) {
        match fs::write(&path, &self.content) {
            Ok(_) => {
                self.current_file = Some(path);
                self.is_modified = false;
            }
            Err(e) => {
                eprintln!("Failed to save file: {}", e);
            }
        }
    }

    fn get_title(&self) -> String {
        let filename = self.current_file
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled");
        
        let modified = if self.is_modified { "*" } else { "" };
        format!("{}{} - rpad", modified, filename)
    }

    fn find_and_replace(&mut self) {
        if !self.find_text.is_empty() && !self.replace_text.is_empty() {
            let new_content = self.content.replace(&self.find_text, &self.replace_text);
            if new_content != self.content {
                self.content = new_content;
                self.is_modified = true;
            }
        }
    }
}

impl eframe::App for RpadApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Ok(serialized) = serde_json::to_string(self) {
            storage.set_string(eframe::APP_KEY, serialized);
        }
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Set window title
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.get_title()));

        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New\tCtrl+N").clicked() {
                        self.new_file();
                        ui.close_menu();
                    }
                    if ui.button("Open...\tCtrl+O").clicked() {
                        self.open_file();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Save\tCtrl+S").clicked() {
                        self.save_file();
                        ui.close_menu();
                    }
                    if ui.button("Save As...\tCtrl+Shift+S").clicked() {
                        self.save_as_file();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Find & Replace\tCtrl+H").clicked() {
                        self.show_find_replace = true;
                        ui.close_menu();
                    }
                });

                ui.menu_button("Format", |ui| {
                    ui.checkbox(&mut self.word_wrap, "Word Wrap");
                    ui.separator();
                    ui.label("Font Size:");
                    ui.add(egui::Slider::new(&mut self.font_size, 8.0..=32.0));
                });

                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.status_bar, "Status Bar");
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About rpad").clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    }
                });
            });
        });

        // Find & Replace dialog
        if self.show_find_replace {
            egui::Window::new("Find & Replace")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Find:");
                        ui.text_edit_singleline(&mut self.find_text);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Replace:");
                        ui.text_edit_singleline(&mut self.replace_text);
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Replace All").clicked() {
                            self.find_and_replace();
                        }
                        if ui.button("Close").clicked() {
                            self.show_find_replace = false;
                        }
                    });
                });
        }

        // About dialog
        if self.show_about {
            egui::Window::new("About rpad")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("rpad - Basic GUI based text editor");
                        ui.label("Version 1.0");
                        ui.label("Built with Rust and egui by Dr. Suresh Ramasamy (https://github.com/sureshdr/rpad)");
                        ui.separator();
                        if ui.button("OK").clicked() {
                            self.show_about = false;
                        }
                    });
                });
        }

        // Status bar
        if self.status_bar {
            egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let lines = self.content.lines().count();
                    let chars = self.content.chars().count();
                    ui.label(format!("Lines: {} | Characters: {}", lines, chars));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.is_modified {
                            ui.label("Modified");
                        } else {
                            ui.label("Ready");
                        }
                    });
                });
            });
        }

        // Main text editor
        egui::CentralPanel::default().show(ctx, |ui| {
            let available_rect = ui.available_rect_before_wrap();
            
            let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                let mut layout_job = egui::text::LayoutJob::default();
                layout_job.append(
                    string,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::monospace(self.font_size),
                        color: ui.visuals().text_color(),
                        ..Default::default()
                    },
                );
                
                if self.word_wrap {
                    layout_job.wrap.max_width = wrap_width;
                }
                
                ui.fonts(|f| f.layout_job(layout_job))
            };

            let response = ui.add_sized(
                available_rect.size(),
                egui::TextEdit::multiline(&mut self.content)
                    .font(egui::TextStyle::Monospace)
                    .code_editor()
                    .layouter(&mut layouter)
            );

            if response.changed() {
                self.is_modified = true;
            }
        });

        // Keyboard shortcuts
        ctx.input(|i| {
            if i.key_pressed(egui::Key::N) && i.modifiers.ctrl {
                self.new_file();
            }
            if i.key_pressed(egui::Key::O) && i.modifiers.ctrl {
                self.open_file();
            }
            if i.key_pressed(egui::Key::S) && i.modifiers.ctrl {
                if i.modifiers.shift {
                    self.save_as_file();
                } else {
                    self.save_file();
                }
            }
            if i.key_pressed(egui::Key::H) && i.modifiers.ctrl {
                self.show_find_replace = true;
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "rpad",
        options,
        Box::new(|cc| Box::new(RpadApp::new(cc))),
    )
}
