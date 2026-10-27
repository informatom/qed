//! qed's domain logic. Nothing in here knows about Tauri or the UI, so it can
//! be tested with plain `cargo test`.

pub mod block;
pub mod calendar;
pub mod deploy;
pub mod fonts;
pub mod generator;
pub mod history;
pub mod icons;
pub mod images;
pub mod links;
pub mod markdown;
pub mod media;
pub mod moving;
pub mod page;
pub mod page_writer;
pub mod preview;
pub mod sftp;
pub mod site;
pub mod slug;
pub mod target;
pub mod theme;
pub mod video;
