use eframe::egui;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc, Mutex};
use crate::app_logic::execute_cycle;

pub struct GcpvApp {
    pat_file: String,
    output_folder: String,
    interval_seconds: u64,
    running: bool,
    last_run: Option<Instant>,
    status_message: String,
    // Thread handling
    is_processing: Arc<Mutex<bool>>,
}

impl Default for GcpvApp {
    fn default() -> Self {
        Self {
            pat_file: "".to_owned(),
            output_folder: "".to_owned(),
            interval_seconds: 60,
            running: false,
            last_run: None,
            status_message: "Ready".to_owned(),
            is_processing: Arc::new(Mutex::new(false)),
        }
    }
}

impl GcpvApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state if using persistence (not implemented yet).
        Self::default()
    }

    fn run_conversion(&mut self) {
        let pat_file = self.pat_file.clone();
        let output_folder = self.output_folder.clone();
        let is_processing = self.is_processing.clone();

        // Update status immediately?
        // We can't easily update self.status_message from thread without a channel or Arc<Mutex>.
        // For simplicity, we just set a flag.
        
        {
            let mut processing = is_processing.lock().unwrap();
            if *processing {
                return; // Already running
            }
            *processing = true;
        }

        thread::spawn(move || {
            let evt_path = PathBuf::from(&output_folder).join("LYNX.EVT");
            let json_path = PathBuf::from(&output_folder).join("races.json");
            
            // We can't access env vars easily if they were relied upon for competition ID overrides,
            // strictly speaking the GUI should probably expose that if needed. 
            // For now passing None for env_competition_id.
            
            let result = execute_cycle(
                &pat_file, 
                evt_path.to_str().unwrap(), 
                json_path.to_str().unwrap(), 
                None
            );

            // Access lock to finish
            let mut processing = is_processing.lock().unwrap();
            *processing = false;
            
            // In a real app we might send the result back via a channel to display error/success.
            // For now just print to stdout
             match result {
                Ok(_) => println!("Conversion successful"),
                Err(e) => eprintln!("Conversion failed: {:?}", e),
            }
        });
    }
}

impl eframe::App for GcpvApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("GCPV Lynx Export");
            
            ui.add_space(10.0);

            // File Selection
            ui.horizontal(|ui| {
                ui.label("PAT File:");
                ui.text_edit_singleline(&mut self.pat_file);
                if ui.button("Select...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().add_filter("PAT", &["pat"]).pick_file() {
                        self.pat_file = path.display().to_string();
                    }
                }
            });

            // Output Folder Selection
            ui.horizontal(|ui| {
                ui.label("Output Folder:");
                ui.text_edit_singleline(&mut self.output_folder);
                if ui.button("Select...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.output_folder = path.display().to_string();
                    }
                }
            });

            // Interval
            ui.horizontal(|ui| {
                ui.label("Interval (seconds):");
                ui.add(egui::DragValue::new(&mut self.interval_seconds).range(1..=3600));
            });

            ui.add_space(20.0);

            // Start/Stop
            ui.horizontal(|ui| {
                if self.running {
                    if ui.button("Stop").clicked() {
                        self.running = false;
                        self.status_message = "Stopped".to_string();
                    }
                    ui.spinner();
                } else {
                    if ui.button("Start").clicked() {
                        if self.pat_file.is_empty() || self.output_folder.is_empty() {
                            self.status_message = "Error: Please select file and output folder".to_string();
                        } else {
                            self.running = true;
                            self.status_message = "Running...".to_string();
                            self.last_run = None; // Trigger immediate run
                        }
                    }
                }
            });

            ui.label(&self.status_message);
            
            // Check processing status
            let is_processing = *self.is_processing.lock().unwrap();
            if is_processing {
                ui.label("Processing...");
            }

            // Background Logic
            if self.running {
                let now = Instant::now();
                let should_run = match self.last_run {
                    Some(last) => now.duration_since(last) >= Duration::from_secs(self.interval_seconds),
                    None => true,
                };

                if should_run && !is_processing {
                    self.last_run = Some(now);
                    self.run_conversion();
                }
                
                // Request repaint to keep checking time/status
                ctx.request_repaint_after(Duration::from_millis(100)); // 10fps check
            }
        });
    }
}
