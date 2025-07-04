use egui::TextureHandle;
use std::path::{PathBuf, Path};
use crate::core::image_cache::ImageCache;
use crate::core::preload_manager::PreloadManager;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use crate::core::file_manager::FileManager;
use crate::ui::navigation_manager::NavigationManager;
use crate::i18n::es::*;
use crate::config::config::*;
use eframe::egui;
use egui::Widget;
use rfd::FileDialog;

pub struct ImageViewerApp {
    // image_cache: ImageCache,
    navigation: NavigationManager,
    // preload_manager: PreloadManager,
    fullscreen: bool,
    image_dir: Option<PathBuf>,
    show_about: bool,
    logo_texture: Option<egui::TextureHandle>,
    current_texture: Option<egui::TextureHandle>,
    texture_receiver: Option<Receiver<egui::TextureHandle>>,
}

impl Default for ImageViewerApp {
    
    fn default() -> Self {
        let (_s, r) = mpsc::channel();
        Self {
            // image_cache: ImageCache::new(2),
            navigation: NavigationManager::new(),
            // preload_manager: PreloadManager::new(2),
            fullscreen: false,
            image_dir: None,
            show_about: false,
            logo_texture: None,
            current_texture: None,
            texture_receiver: Some(r),
        }
    }
}

impl ImageViewerApp {
    fn load_images_from_folder(&mut self, ctx: &egui::Context, path: &Path) {
        if let Some(parent) = path.parent() {
            // self.image_cache.clear(); // ❌ ya no usamos cache

            match FileManager::scan_directory(parent) {
                Ok(image_paths) => {
                    let initial_index = FileManager::find_index_of_file(&image_paths, path).unwrap_or(0);
                    self.navigation.set_images(image_paths, initial_index);
                    self.image_dir = Some(parent.to_path_buf());
                    self.load_current_image_async(ctx);
                }
                Err(e) => {
                    eprintln!("Error escaneando directorio: {}", e);
                }
            }
        }
    }

    pub fn init(&mut self, ctx: &egui::Context) {
        self.logo_texture = Some(Self::load_logo_texture(ctx));
    }

    fn load_logo_texture(ctx: &egui::Context) -> egui::TextureHandle {
        use egui::{ColorImage, TextureOptions};
        let bytes = include_bytes!("../../assets/images/logo.png");
        let image = image::load_from_memory(bytes).unwrap().to_rgba8();
        let size = [image.width() as usize, image.height() as usize];
        let color_image = ColorImage::from_rgba_unmultiplied(size, image.as_raw());
        ctx.load_texture("logo", color_image, TextureOptions::default())
    }

    fn next_image(&mut self, ctx: &egui::Context) {
    if self.navigation.next() {
        self.load_current_image_async(ctx);
    }
}

fn previous_image(&mut self, ctx: &egui::Context) {
    if self.navigation.previous() {
        self.load_current_image_async(ctx);
    }
}

    // fn preload_images(&self, ctx: &egui::Context) {
    //     self.preload_manager.preload_images_around_index(
    //         self.navigation.image_paths(),
    //         self.navigation.current_index(),
    //         &self.image_cache
    //     );
    // }

    fn load_current_image_async(&mut self, ctx: &egui::Context) {
    use std::fs;
    use egui::{ColorImage, TextureOptions};
    use std::thread;

    if let Some(path) = self.navigation.current_path() {
        let path = path.to_path_buf();
        let ctx = ctx.clone();

        let (tx, rx) = mpsc::channel();
        self.texture_receiver = Some(rx);

        thread::spawn(move || {
            if let Ok(bytes) = fs::read(path) {
                if let Ok(image) = image::load_from_memory(&bytes) {
                    let rgba = image.to_rgba8();
                    let size = [rgba.width() as usize, rgba.height() as usize];
                    let color_image = ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());

                    // cargar en el hilo principal más tarde
                    let texture = ctx.load_texture("current_image", color_image, TextureOptions::default());
                    let _ = tx.send(texture);
                }
            }
        });
    }
}


    fn get_current_texture(&self) -> Option<&TextureHandle> {
        self.current_texture.as_ref()
    }

    fn handle_keyboard_input(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Escape) && self.fullscreen {
                let ctx = ctx.clone();
                std::thread::spawn(move || {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                });
                self.fullscreen = false;
            }
            if i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::Space) {
                self.next_image(ctx);
            }
            if i.key_pressed(egui::Key::ArrowLeft) {
                self.previous_image(ctx);
            }
            if i.key_pressed(egui::Key::F11) {
                let new_fullscreen = !self.fullscreen;
                let ctx = ctx.clone();
                std::thread::spawn(move || {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(new_fullscreen));
                });
                self.fullscreen = new_fullscreen;
            }
        });
    }

    fn show_toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.show_file_controls(ui, ctx);
                ui.separator();
                self.show_navigation_controls(ui, ctx);
                ui.separator();
                self.show_view_controls(ui, ctx);
                self.show_image_info(ui);
            });
        });
    }

    fn show_file_controls(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if ui.button(format!("📂 {}", BTN_OPEN)).clicked() {
            let (filter_name, extensions) = FileManager::get_supported_file_filter();
            if let Some(path) = FileDialog::new()
                .add_filter(filter_name, &extensions)
                .pick_file()
            {
                self.load_images_from_folder(ctx, &path);
            }
        }
    }

    fn show_navigation_controls(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let can_go_prev = self.navigation.can_go_previous();
        let can_go_next = self.navigation.can_go_next();

        if ui.add_enabled(can_go_prev, egui::Button::new(format!("⬅️{}", BTN_PREV))).clicked() {
            self.previous_image(ctx);
        }

        if ui.add_enabled(can_go_next, egui::Button::new(format!("➡️ {}", BTN_NEXT))).clicked() {
            self.next_image(ctx);
        }
    }

    fn show_view_controls(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if ui.button(format!("🖥️ {}", BTN_FULLSCREEN)).clicked() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
            self.fullscreen = true;
        }

        if ui.button(format!("ℹ️ {}", BTN_ABOUT)).clicked() {
            self.show_about = true;
        }
    }

    fn show_image_info(&self, ui: &mut egui::Ui) {
        if !self.navigation.is_empty() {
            ui.separator();
            ui.label(format!("{} / {}", 
                self.navigation.current_index() + 1, 
                self.navigation.total_images()
            ));

            if let Some(current_path) = self.navigation.current_path() {
                if let Some(filename) = current_path.file_name() {
                    ui.label(filename.to_string_lossy());
                }
            }
        }
    }

    fn show_about_dialog(&mut self, ctx: &egui::Context) {
        if self.show_about {
            if self.logo_texture.is_none() {
                self.logo_texture = Some(Self::load_logo_texture(ctx));
            }

            egui::Window::new(BTN_ABOUT)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .min_width(300.0)
                .min_height(250.0)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        if let Some(logo_texture) = &self.logo_texture {
                            let logo_size = egui::Vec2::new(128.0, 128.0);
                            egui::Image::from_texture(logo_texture)
                                .fit_to_exact_size(logo_size)
                                .ui(ui);
                        } else {
                            ui.label(ERROR_LOGO);
                        }

                        ui.add_space(10.0);
                        ui.label(APP_NAME);
                        ui.label(format!("v{}", APP_VERSION));
                        ui.label(format!("{}{}", TEXT_AUTHOR, APP_AUTHOR));
                        ui.label(TEXT_LICENSE);
                        ui.label(TEXT_INFOAPP);
                    });

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                        if ui.button(BTN_CLOSE).clicked() {
                            self.show_about = false;
                        }
                    });
                });
        }
    }

    fn show_main_content(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let texture = self.get_current_texture().cloned();
            if let Some(texture) = texture {
                self.render_image(ui, &texture);
            } else {
                self.show_placeholder_text(ui);
            }
        });
    }

    fn render_image(&self, ui: &mut egui::Ui, texture: &TextureHandle) {
        let available_size = ui.available_size();
        let image_size = texture.size_vec2();

        let scale = (available_size.x / image_size.x)
            .min(available_size.y / image_size.y)
            .min(1.0);

        let scaled_size = image_size * scale;

        let rect = egui::Rect::from_center_size(
            ui.available_rect_before_wrap().center(),
            scaled_size,
        );

        let response = ui.allocate_rect(rect, egui::Sense::hover());
        egui::Image::from_texture(texture)
            .fit_to_exact_size(scaled_size)
            .paint_at(ui, response.rect);
    }

    fn show_placeholder_text(&self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            if self.navigation.is_empty() {
                ui.label("");
                ui.label("");
                ui.label(TEXT_OPENIMG);
                ui.label(TEXT_ROWSORSPACE);
                ui.label(TEXT_F11FULLSCREEN);
            } else {
                ui.label(TEXT_LOADINGIMG);
            }
        });
    }
    // ... previous methods ...

    // pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    //     ctx.request_repaint();

    //     self.handle_keyboard_input(ctx);

    //     // Verificar si llegó una textura nueva
    //     if let Some(receiver) = &self.texture_receiver {
    //         if let Ok(texture) = receiver.try_recv() {
    //             self.current_texture = Some(texture);
    //         }
    //     }

    //     if !self.fullscreen {
    //         self.show_toolbar(ctx);
    //     }

    //     self.show_about_dialog(ctx);
    //     self.show_main_content(ctx);
    // }
}

impl eframe::App for ImageViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        self.handle_keyboard_input(ctx);

        // Verificar si llegó una textura nueva
        if let Some(receiver) = &self.texture_receiver {
            if let Ok(texture) = receiver.try_recv() {
                self.current_texture = Some(texture);
            }
        }

        if !self.fullscreen {
            self.show_toolbar(ctx);
        }

        self.show_about_dialog(ctx);
        self.show_main_content(ctx);
    }
}
