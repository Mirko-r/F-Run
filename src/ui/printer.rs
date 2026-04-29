//! Funzioni di stampa e logging
//!
//! Fornisce utility per stampare messaggi sul terminale
//! con colori e simboli, visualizzare banner, messaggi di errore, warning
//! o successo, e mostrare informazioni sui progetti Flutter.
//!

use crate::{
    core::{pubspec::Pubspec},
    ui::colors::{GREEN, RED, RESET, YELLOW},
};
use figlet_rs::FIGfont;
use std::process;

/// Stampa messaggio colorato
///
/// # Parametri
/// - `icon`: Icona da mostrare prima e dopo il messaggio
/// - `color`: Colore del messaggio
/// - `msg`: Messaggio da stampare
fn print_msg(icon: &str, color: &str, msg: &str) {
    println!("{color}{icon} {msg} {icon}{RESET}");
}

/// Stampa un errore e ritorna un codice
/// # Parametri
/// - `message`: Messaggio di errore
/// - `code`: Codice di errore
pub fn error_and_exit(message: &str, code: i32) -> ! {
    eprintln!("{RED}❌ Error: {message} ❌{RESET}");
    process::exit(code);
}

/// Stampa un errore
/// # Parametri
/// - `message`: Messaggio di errore
pub fn error(message: &str) {
    eprintln!("{RED}❌ Error: {message} ❌{RESET}");
}

/// Stampa un sucesso
/// # Parametri
/// - `message`: Messaggio di successo
pub fn ok(message: &str) {
    print_msg("✨ 🎉 ✨", GREEN, message);
}

/// Stampa un warning
/// # Parametri
/// - `message`: Messaggio di warning
pub fn warn(message: &str) {
    print_msg("⚠️", YELLOW, message);
}

/// Stampa il banner con nome e versione
///
/// # Panics
/// - Termina il programma se non pè possibile inizializzare `FIGfont`
/// - Termina il programma se non è possibile convertire convertire il font.
pub fn banner() {
    let font: FIGfont = FIGfont::standard().expect("Impossibile inizializzare FIGfont");

    let banner_text = "F-RUN";

    if let Some(figure) = font.convert(banner_text) {
        let figure_str = figure.to_string();
        let max_w = figure_str.lines().map(str::len).max().unwrap_or(0);

        // Stampa centrata
        for line in figure_str.lines() {
            println!("{line:^max_w$}");
        }

        // Messaggio di benvenuto centrato rispetto al banner
        let welcome_msg = "Benvenuto!".to_string();

        println!("{welcome_msg:^max_w$}");
        println!();

        // Versione
        let version_raw = format!("v{}", env!("CARGO_PKG_VERSION"));

        // Calcolo spazio a sinistra
        // saturating_sub evita numeri negativi, / 2 centra.
        let left_padding = max_w.saturating_sub(version_raw.len()) / 2;

        // Spazi a sinistra + Versione Colorata
        println!(
            "{:indent$}{}{}{}",
            "",
            GREEN,
            version_raw,
            RESET,
            indent = left_padding
        );
    }
}

/// Stampa le info del progetto Flutter
///
/// # Parametri
/// - `pubspec`: riferimento alla configurazione del progetto Flutter [`crate::core::pubspec::Pubspec`]
pub fn project_info(pubspec: &Pubspec) {
    println!();
    println!(
        "📦 {}{}",
        pubspec.name,
        pubspec
            .version
            .as_ref()
            .map(|v| format!(" ({v})"))
            .unwrap_or_default()
    );

    if let Some(desc) = &pubspec.description {
        println!("📝 {desc}");
    }
}
