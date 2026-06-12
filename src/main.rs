use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let base_dir = args.get(1).map(|s| s.as_str()).unwrap_or(".");

    if let Err(e) = wilde_ssg::builder::build_site(Path::new(base_dir)) {
        eprintln!("Build failed: {e}");
        std::process::exit(1);
    }
}
