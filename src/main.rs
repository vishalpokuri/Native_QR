use eframe::egui::{self};
use image;
use rdev::{listen, Button, Event, EventType};
use rqrr::PreparedImage;
use screenshots::Screen;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use webbrowser;

#[derive(Debug)]
struct Coordinates {
    x: f32,
    y: f32,
}
struct QRScanner {
    scanning: bool,
    start: bool,
    result: String,
    tx: mpsc::Sender<(i32, i32, u32, u32)>,
    rx: mpsc::Receiver<Vec<u8>>,
    start_pos: Option<Coordinates>,
    end_pos: Option<Coordinates>,
}

impl Default for QRScanner {
    fn default() -> Self {
        let (tx1, rx1) = mpsc::channel();
        let (_tx2, rx2) = mpsc::channel();

        // Spawn the background thread for screen capture
        thread::spawn(move || {
            let screen = Screen::from_point(0, 0).unwrap();
            loop {
                if let Ok((x, y, width, height)) = rx1.recv() {
                    let imageresult = screen.capture_area(x, y, width, height).unwrap();
                    println!("{:?}", imageresult);
                    let imageresult = imageresult.to_vec();
                    let decoded_data_resultant = decode_qr_code(imageresult);
                    println!("{:?}", decoded_data_resultant.unwrap());
                }
            }
        });

        Self {
            start: false,
            scanning: false,
            result: String::new(),
            tx: tx1,
            rx: rx2,
            start_pos: None,
            end_pos: None,
        }
    }
}
impl QRScanner {
    fn handle_event(&mut self, event: Event) {
        match event.event_type {
            EventType::MouseMove { x, y } => {
                if !self.scanning {
                    // Update start_pos only when not scanning
                    self.start_pos = Some(Coordinates {
                        x: x as f32,
                        y: y as f32,
                    });
                } else {
                    // Update end_pos when scanning (dragging)
                    self.end_pos = Some(Coordinates {
                        x: x as f32,
                        y: y as f32,
                    });
                }
                println!("{:?} {:?}", self.start_pos, self.end_pos);
            }
            EventType::ButtonPress(Button::Left) => {
                if !self.scanning {
                    self.scanning = true;
                    // Use the current mouse position as both start and end initially
                }
            }
            EventType::ButtonRelease(Button::Left) => {
                if self.scanning {
                    self.scanning = false;

                    // Process the captured area
                    if let (Some(start), Some(end)) = (&self.start_pos, &self.end_pos) {
                        let width = (end.x - start.x).abs() as u32;
                        let height = (end.y - start.y).abs() as u32;
                        let (x, y) = (start.x.min(end.x) as i32, start.y.min(end.y) as i32);

                        self.tx.send((x, y, width, height)).unwrap();
                    }
                }
            }
            _ => (),
        }
    }
}

impl eframe::App for QRScanner {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.start {
                if ui.button("Scan QR Code").clicked() {
                    self.start = true;
                    self.scanning = false;
                    self.result.clear();
                    self.start_pos = None;
                    self.end_pos = None;
                    println!("Started scanning: {:?}", self.start);
                }
            } else {
                ui.label("Drag to select area");
            }

            if let Ok(_image_data) = self.rx.try_recv() {
                // We'll process the image data in the next step
                self.result = "Image captured".to_string();
            }

            if !self.result.is_empty() {
                ui.label(&self.result);
            }

            // println!("{:?}", &self.end_pos);
        });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();
    let qr_scanner = Arc::new(Mutex::new(QRScanner::default()));

    let qr_scanner_clone = Arc::clone(&qr_scanner);
    thread::spawn(move || {
        let callback = move |event: Event| {
            if let Ok(mut scanner) = qr_scanner_clone.lock() {
                scanner.handle_event(event);
            }
        };

        if let Err(error) = listen(callback) {
            println!("Error: {:?}", error)
        }
    });

    eframe::run_native(
        "QR Scanner",
        options,
        Box::new(|_cc| Ok(Box::new(QRScanner::default()))),
    )
}

fn decode_qr_code(image: Vec<u8>) -> Option<String> {
    let img = image::load_from_memory(&image).ok().unwrap().to_luma8();
    let mut prepared_image = PreparedImage::prepare(img);

    // Detect and decode QR codes
    let grids = prepared_image.detect_grids();

    let (_meta, content) = grids[0].decode().unwrap();
    if !content.is_empty() {
        webbrowser::open(&content).unwrap();
        return Some(content);
    } else {
        None
    }
}
