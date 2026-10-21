//! Builds a site folder from the command line, without the app:
//!
//! ```sh
//! cargo run -p qed-core --example build -- docs
//! ```

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let Some(folder) = std::env::args().nth(1).map(PathBuf::from) else {
        eprintln!("usage: build <site folder>");
        return ExitCode::FAILURE;
    };

    let result = qed_core::site::open(&folder)
        .map_err(|e| e.to_string())
        .and_then(|site| qed_core::generator::build(&site).map_err(|e| e.to_string()));

    match result {
        Ok(report) => {
            println!(
                "Built {} pages, {} redirects, {} media files and {} resized images into {}",
                report.pages,
                report.redirects,
                report.media_files,
                report.resized_images,
                report.out_dir.display()
            );
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
