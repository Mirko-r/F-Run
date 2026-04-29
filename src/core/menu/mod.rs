//! Modulo root per la gestione dei menu CLI interattivi.
//!
//! Espone:
//! - menu principali e generatori (`main_menu`, `generators_menu`)
//! - tema grafico condiviso (`menu_theme`)
//! - builder e logica di rendering per menu ricercabili (`searchable_menu`)
//! - funzioni di orchestrazione e shortcut (`menus`)
//!
//! Ogni sottomodulo isola una responsabilità verticale della CLI.

pub mod generators_menu;
pub mod main_menu;
pub mod menu_theme;
pub mod menus;
pub mod searchable_menu;
