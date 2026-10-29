//! The manual's pages and pictures are compiled in (see `manual.rs`); this
//! tells Cargo to rebuild when they change, which it doesn't notice by
//! itself for files read by a macro.

fn main() {
    println!("cargo:rerun-if-changed=../../docs/pages/manual");
    println!("cargo:rerun-if-changed=../../docs/media/manual");
}
