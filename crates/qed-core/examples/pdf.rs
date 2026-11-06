//! Makes the PDFs of a site folder from the command line, without the app
//! (#154): one per language, in the site's `assets/`.
//!
//! ```sh
//! cargo run -p qed-core --example pdf -- docs
//! cargo run -p qed-core --example pdf -- docs --typst > book.typ
//! ```
//!
//! With `--typst`, it prints the Typst markup of the main language's PDF
//! instead of writing anything.

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(folder) = args.next().map(PathBuf::from) else {
        eprintln!("usage: pdf <site folder> [--typst]");
        return ExitCode::FAILURE;
    };
    let site = match qed_core::site::open(&folder) {
        Ok(site) => site,
        Err(problem) => {
            eprintln!("{problem}");
            return ExitCode::FAILURE;
        }
    };
    if args.next().as_deref() == Some("--typst") {
        print!("{}", qed_core::pdf::markup(&site, None));
        return ExitCode::SUCCESS;
    }
    match qed_core::pdf::export(&site) {
        Ok(report) => {
            for file in &report.files {
                println!(
                    "assets/{}: {} pages, {:.1} MB",
                    file.name,
                    file.pages,
                    f64::from(file.size) / 1_000_000.0
                );
            }
            for note in &report.notes {
                println!("{note}");
            }
            ExitCode::SUCCESS
        }
        Err(problem) => {
            eprintln!("{problem}");
            ExitCode::FAILURE
        }
    }
}
