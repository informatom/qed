//! qed's domain logic. Nothing in here knows about Tauri or the UI, so it can
//! be tested with plain `cargo test`.

pub mod block;
pub mod page;
pub mod site;
pub mod slug;
