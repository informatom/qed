//! qed's domain logic. Nothing in here knows about Tauri or the UI, so it can
//! be tested with plain `cargo test`.

pub mod block;
pub mod generator;
pub mod links;
pub mod markdown;
pub mod page;
pub mod page_writer;
pub mod preview;
pub mod site;
pub mod slug;
