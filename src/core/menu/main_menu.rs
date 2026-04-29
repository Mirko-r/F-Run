/// Modulo per il menu principale della CLI.
///
/// Usa il menu nativo del progetto con estetica modern e shortcut numeriche.
use crate::{
    config::frunconfig::FrunConfig,
    core::menu::{menu_theme::get_theme, searchable_menu::menu::searchable_menu},
};

/// Azioni selezionabili dal menu principale della CLI.
#[derive(Clone, Copy)]
pub enum MainMenuAction {
    Clean,
    CleanF,
    Generate,
    BuildRunner,
    ShorebirdPatch,
    Build,
    Advanced,
    Exit,
}

/// Mostra il menu principale e ritorna l'azione selezionata.
///
/// # Return
/// Azione selezionata dall'utente (`MainMenuAction`).
pub fn show_main_menu() -> MainMenuAction {
    let mut items: Vec<(&str, MainMenuAction)> = vec![
        ("Pulizia", MainMenuAction::Clean),
        ("Pulizia forzata", MainMenuAction::CleanF),
        ("Build Runner", MainMenuAction::BuildRunner),
    ];

    if let Some(cfg) = FrunConfig::get() {
        if cfg.has_generators() {
            items.push(("Genera", MainMenuAction::Generate));
        }
        if cfg.features.shorebird {
            items.push(("Shorebird Patch", MainMenuAction::ShorebirdPatch));
        }
    }

    items.extend([
        ("Build", MainMenuAction::Build),
        ("Avanzate", MainMenuAction::Advanced),
        ("Esci", MainMenuAction::Exit),
    ]);

    // label numerate: "1. foo", "2. bar", ecc.
    let numbered_labels: Vec<String> = items
        .iter()
        .enumerate()
        .map(|(i, (label, _))| format!("{}. {}", i + 1, label))
        .collect();

    println!();

    // Show menu
    let selection_result = searchable_menu(" MENU PRINCIPALE", numbered_labels.clone())
        .with_theme(&get_theme())
        .with_page_size(12)
        .with_help_message("Usa i numeri come shortcut • Esc per uscire")
        .prompt();

    selection_result.map_or(MainMenuAction::Exit, |choice| {
        // Search indice della stringa selezionata nella lista numerata
        let index = numbered_labels
            .iter()
            .position(|s| s == &choice)
            .unwrap_or(items.len() - 1);

        items[index].1
    })
}
