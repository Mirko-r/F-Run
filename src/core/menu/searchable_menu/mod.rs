//! Menu selezionabile con filtro incrementale, navigazione da tastiera e auto-submit.
//!
//! Questo modulo espone un builder (`SearchableMenu`) e separa:
//! - errori e tipo risultato
//! - gestione raw mode/input terminale
//! - rendering della mini-UI del prompt

mod error;
pub mod menu;
mod render;
mod terminal;
