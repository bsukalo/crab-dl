mod downloader;
mod gui;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&"--gui".to_string()) {
        gui::run_gui().unwrap();
    } else {
        downloader::run_cli();
    }
}
