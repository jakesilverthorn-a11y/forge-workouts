#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#![allow(dead_code)]

mod app;
mod body;
mod charts;
mod data;
mod engine;
mod theme;
mod util;

use eframe::egui;

/// Web build: draw into the `<canvas id="the_canvas_id">` in index.html.
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("no window")
            .document()
            .expect("no document");
        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("missing the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id is not a canvas");

        let result = eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| Ok(Box::new(app::ForgeApp::new(cc, None)))),
            )
            .await;

        if let Some(loading) = document.get_element_by_id("loading_text") {
            match result {
                Ok(_) => loading.remove(),
                Err(e) => {
                    loading.set_inner_html("<p>Forge couldn't start in this browser. See the developer console for details.</p>");
                    panic!("failed to start eframe: {e:?}");
                }
            }
        }
    });
}

/// Desktop build.
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // `--screenshot <file.png>` (or `--screenshot=<file.png>`): draw ~120 frames, save a PNG, exit.
    let args: Vec<String> = std::env::args().collect();
    let mut shot: Option<std::path::PathBuf> = None;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--screenshot" {
            if let Some(p) = args.get(i + 1) {
                shot = Some(std::path::PathBuf::from(p));
                i += 1;
            }
        } else if let Some(rest) = args[i].strip_prefix("--screenshot=") {
            shot = Some(std::path::PathBuf::from(rest));
        }
        i += 1;
    }

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Forge - Smart Workout Planner")
        .with_inner_size([1280.0, 820.0])
        .with_min_inner_size([980.0, 640.0]);

    let mut options = eframe::NativeOptions::default();

    if shot.is_some() {
        // Never steal focus from the person while a check runs.
        viewport = viewport.with_active(false);
        #[cfg(windows)]
        {
            viewport = viewport.with_window_level(egui::WindowLevel::AlwaysOnBottom);
        }
        #[cfg(target_os = "macos")]
        {
            options.event_loop_builder = Some(Box::new(|builder| {
                use winit::platform::macos::EventLoopBuilderExtMacOS;
                builder.with_activate_ignoring_other_apps(false);
            }));
        }
        options.persist_window = false;
    }
    options.viewport = viewport;

    eframe::run_native(
        "Forge Workouts",
        options,
        Box::new(move |cc| Ok(Box::new(app::ForgeApp::new(cc, shot)))),
    )
}
