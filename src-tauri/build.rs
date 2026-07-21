use tauri_build::Config;

fn main() {
    tauri_build::Builder::default()
        .config(Config::default())
        .build();
}
