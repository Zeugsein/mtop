fn main() {
    if let Err(e) = mtop::tui::run_stories() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
