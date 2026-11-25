# Forge

A smart workout planner that adapts each day's workout to how recovered your muscles are. Built in Rust with [egui](https://github.com/emilk/egui), it runs as a desktop app and in the browser.

## Run it

- **In the browser:** every push to `main` is built and published to GitHub Pages by `.github/workflows/pages.yml`.
- **On the desktop:** `cargo run --release`
- **Web version locally:** install [Trunk](https://trunkrs.dev) and the wasm target (`rustup target add wasm32-unknown-unknown`), then run `trunk serve` and open http://127.0.0.1:8080.

Your workouts are saved in the browser's local storage (web) or in the app's settings folder (desktop). They stay on your device.
