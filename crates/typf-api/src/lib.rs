// this_file: crates/typf-api/src/lib.rs

//! The public API for the TYPF text rendering engine.
//!
//! This crate provides a high-level, unified interface for text shaping and
//! rendering across various backend implementations (e.g., HarfBuzz, CoreText, Orge).
//!
//! # Getting Started
//!
//! ```no_run
//! use typf_api::{Session, SessionBuilder, Backend};
//! use typf_fontdb::{FontDatabase, Query};
//! use std::path::PathBuf;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 1. Initialize a FontDatabase and load fonts
//!     let mut font_db = FontDatabase::new();
//!     font_db.load_fonts_dir(PathBuf::from("/path/to/my/fonts"));
//!
//!     // 2. Query for a specific font
//!     let font_query = Query {
//!         family: "Noto Sans".to_string(),
//!         ..Default::default()
//!     };
//!     let font = font_db.query(&font_query)
//!         .ok_or("Font not found")?;
//!
//!     // 3. Create a Session using the builder
//!     let session = SessionBuilder::new(&font_db, font)
//!         .with_backend(Backend::HarfBuzz) // Explicitly choose a backend
//!         .build();
//!
//!     // 4. Render some text
//!     let text = "Hello, TYPF!";
//!     let bitmap = session.render(text, 24.0, None, false)?;
//!
//!     // 5. Save the output (e.g., to a PNG file)
//!     bitmap.save_png("output.png")?;
//!
//!     println!("Text rendered to output.png");
//!     Ok(())
//! }
//! ```

mod backend; // Add this
mod session;

pub use backend::{Backend, BackendFeatures, DynBackend, Point, FontMetrics, create_backend, create_default_backend}; // Update this
pub use session::{Session, SessionBuilder};