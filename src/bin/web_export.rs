use eframe::egui;
use axum::{
    extract::State,
    routing::get,
    Router,
    response::{IntoResponse, Json},
    http::Method,
};
use tower_http::cors::CorsLayer;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use rust_gcpv_lynx_export::app_logic::fetch_race_data;
use rust_gcpv_lynx_export::writer::generate_race_json;
use rust_gcpv_lynx_export::writer::JsonRace;

// Shared state for the web server
type SharedState = Arc<RwLock<Vec<JsonRace>>>;

#[derive(Clone)]
struct AppState {
    data: SharedState,
}

// Web Server Logic
async fn get_races(State(state): State<AppState>) -> impl IntoResponse {
    let data = state.data.read().unwrap();
    Json(data.clone())
}

async fn run_server(port: u16, state: SharedState) {
    let app_state = AppState { data: state };
    
    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        .allow_origin(tower_http::cors::Any);

    let app = Router::new()
        .route("/races", get(get_races))
        .layer(cors)
        .with_state(app_state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    println!("Listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// GUI Application
struct WebApp {
    pat_file: String,
    port: u16,
    interval_seconds: u64,
    running: bool,
    status_message: String,
    // State
    shared_data: SharedState,
    last_run: Option<Instant>,
    // Tokio Runtime for server
    runtime: Option<Runtime>,
}

impl Default for WebApp {
    fn default() -> Self {
        Self {
            pat_file: "".to_owned(),
            port: 3030,
            interval_seconds: 5,
            running: false,
            status_message: "Ready".to_owned(),
            shared_data: Arc::new(RwLock::new(Vec::new())),
            last_run: None,
            runtime: None,
        }
    }
}

impl WebApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn start_server(&mut self) {
        if self.runtime.is_none() {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            
            let port = self.port;
            let state = self.shared_data.clone();
            
            rt.spawn(async move {
                run_server(port, state).await;
            });
            
            self.runtime = Some(rt);
        }
    }

    fn update_data(&mut self) {
        // Fetch data
        let env_id = None; 
        match fetch_race_data(&self.pat_file, env_id) {
            Ok(race_data) => {
                // Generate JSON
                 match generate_race_json(
                    &race_data.races, 
                    &race_data.programs, 
                    &race_data.lanes, 
                    &race_data.competitors, 
                    &race_data.competitors_in_comp
                ) {
                    Ok(json) => {
                        let mut data = self.shared_data.write().unwrap();
                        *data = json;
                    }
                    Err(e) => {
                        eprintln!("Error generating JSON: {:?}", e);
                        self.status_message = format!("Error generating JSON: {}", e);
                    }
                }
            }
            Err(e) => {
                 eprintln!("Error fetching data: {:?}", e);
                 self.status_message = format!("Error fetching data: {}", e);
            }
        }
    }
}

impl eframe::App for WebApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("GCPV Web Exporter");
            ui.add_space(10.0);

            // Inputs
            ui.horizontal(|ui| {
                ui.label("PAT File:");
                ui.text_edit_singleline(&mut self.pat_file);
                if ui.button("Select...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().add_filter("PAT", &["pat"]).pick_file() {
                        self.pat_file = path.display().to_string();
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Port:");
                ui.add(egui::DragValue::new(&mut self.port).range(1024..=65535));
            });

            ui.horizontal(|ui| {
                ui.label("Interval (s):");
                ui.add(egui::DragValue::new(&mut self.interval_seconds).range(1..=3600));
            });

            ui.add_space(20.0);

            // Controls
            if self.running {
                if ui.button("Stop").clicked() {
                    self.running = false;
                    self.status_message = "Stopped (Server may still be running)".to_string();
                    // Note: Stopping tokio runtime gracefully in immediate mode GUI is tricky.
                    // For now we just stop updating the data.
                }
                ui.label(format!("Running on http://localhost:{}/races", self.port));
            } else {
                if ui.button("Start").clicked() {
                     if self.pat_file.is_empty() {
                        self.status_message = "Error: Please select PAT file".to_string();
                    } else {
                        self.running = true;
                        self.status_message = "Starting...".to_string();
                        self.start_server();
                        self.update_data(); // Initial update
                        self.last_run = Some(Instant::now());
                    }
                }
            }
            
            ui.label(&self.status_message);

            // Loop
            if self.running {
                let now = Instant::now();
                 if let Some(last) = self.last_run {
                    if now.duration_since(last) >= Duration::from_secs(self.interval_seconds) {
                        self.update_data();
                        self.last_run = Some(now);
                    }
                }
                ctx.request_repaint_after(Duration::from_millis(100));
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    // Need to initialize generic array or similar? No, standard Vec.
    
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "GCPV Web Export",
        native_options,
        Box::new(|cc| Ok(Box::new(WebApp::new(cc)))),
    )
}
