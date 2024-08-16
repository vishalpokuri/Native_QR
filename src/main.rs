use eframe::egui;
use screenshots::Screen;
use std::sync::mpsc;
use std::thread;

struct QRScanner {
    scanning: bool,
    result: String,
    tx: mpsc::Sender<(i32, i32, u32, u32)>,
    rx: mpsc::Receiver<Vec<u8>>,
    start_pos: Option<egui::Pos2>,
    end_pos: Option<egui::Pos2>,
}

impl Default for QRScanner {
    fn default() -> Self {
        let (tx1, rx1) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();

        // Spawn the background thread for screen capture
        thread::spawn(move || {
            let screen = Screen::from_point(0, 0).unwrap();
            loop {
                if let Ok((x, y, width, height)) = rx1.recv() {
                    let image = screen.capture_area(x, y, width, height).unwrap();
                    //TODO
                    // tx2.send(image.to_vec()).unwrap();
                }
            }
        });

        Self {
            scanning: false,
            result: String::new(),
            tx: tx1,
            rx: rx2,
            start_pos: None,
            end_pos: None,
        }
    }
}

impl eframe::App for QRScanner {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.scanning {
                if ui.button("Scan QR Code").clicked() {
                    self.scanning = true;
                    self.result.clear();
                    self.start_pos = None;
                    self.end_pos = None;
                }
            } else {
                ui.label("Click and drag to select area");
                let pointer = ui.input(|i| i.pointer.clone());
                if pointer.any_pressed() {
                    self.start_pos = pointer.interact_pos();
                    println!("Mouse pressed at: {:?}", self.start_pos);
                }
                if pointer.primary_down() {
                    self.end_pos = pointer.interact_pos();
                    println!("Mouse dragged to: {:?}", self.end_pos);
                }
                if pointer.any_released() {
                    if let (Some(start), Some(end)) = (self.start_pos, self.end_pos) {
                        let min_x = start.x.min(end.x);
                        let min_y = start.y.min(end.y);
                        let width = (start.x - end.x).abs() as u32;
                        let height = (start.y - end.y).abs() as u32;
                        println!(
                            "Selection area: ({}, {}, {}, {})",
                            min_x, min_y, width, height
                        );
                        let tx = self.tx.clone();
                        thread::spawn(move || {
                            tx.send((min_x as i32, min_y as i32, width, height))
                                .unwrap();
                        });
                        self.scanning = false;
                    }
                }

                if let (Some(start), Some(end)) = (self.start_pos, self.end_pos) {
                    let rect = egui::Rect::from_two_pos(start, end);
                    ui.painter()
                        .rect_stroke(rect, 0.0, (1.0, egui::Color32::RED));
                }
            }

            if let Ok(_image_data) = self.rx.try_recv() {
                // We'll process the image data in the next step
                self.result = "Image captured".to_string();
            }

            if !self.result.is_empty() {
                ui.label(&self.result);
            }
        });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Qr Scanner",
        options,
        Box::new(|_cc| Ok(Box::<QRScanner>::default())),
    )
}
