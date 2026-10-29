//! qed's domain logic. Nothing in here knows about Tauri or the UI, so it can
//! be tested with plain `cargo test`.

pub mod block;
pub mod calendar;
pub mod catppuccin_themes;
pub mod chart;
pub mod chem;
pub mod contributors;
pub mod deploy;
pub mod feed;
pub mod fonts;
pub mod generator;
pub mod history;
pub mod icons;
pub mod images;
pub mod links;
pub mod manual;
pub mod markdown;
pub mod media;
pub mod moving;
pub mod page;
pub mod page_writer;
pub mod podcast;
pub mod preview;
pub mod remote;
pub mod search;
pub mod sftp;
pub mod shared;
pub mod site;
pub mod slug;
pub mod strata;
pub mod target;
pub mod theme;
pub mod video;
