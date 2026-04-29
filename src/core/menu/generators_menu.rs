/// Modulo per il menu dei generatori asset.
///
/// Usa il menu nativo del progetto con estetica modern e shortcut numeriche.
///
use crate::{
    config::frunconfig::FrunConfig,
    core::{
        exit_codes::CONFIGERROR,
        menu::{menu_theme::get_theme, searchable_menu::menu::searchable_menu},
    },
    ui::printer::error_and_exit,
};

/// Azioni selezionabili dal menu generatori asset.
#[derive(Clone, Copy)]
pub enum GeneratorsActions {
    Language,
    Icons,
    Splash,
    Exit,
}

/// Mostra il menu generatori asset e ritorna l'azione selezionata.
///
/// # Return
/// Azione selezionata dall'utente (`GeneratorsActions`).
pub fn show_generators_menu() -> GeneratorsActions {
    let mut items: Vec<(&str, GeneratorsActions)> = vec![];

    if let Some(cfg) = FrunConfig::get() {
        if cfg.features.icons_launcher {
            items.push(("Icone", GeneratorsActions::Icons));
        }
        if cfg.features.flutter_native_splash {
            items.push(("Splash", GeneratorsActions::Splash));
        }
        if cfg.features.katana.enabled {
            items.push(("Lingue", GeneratorsActions::Language));
        }
        items.push(("Indietro", GeneratorsActions::Exit));
    } else {
        error_and_exit("Impossibile leggere la configurazione", CONFIGERROR);
    }

    // label numerate: "1. foo", "2. bar", ecc.
    let numbered_labels: Vec<String> = items
        .iter()
        .enumerate()
        .map(|(i, (label, _))| format!("{}. {}", i + 1, label))
        .collect();

    println!();

    // Show menu
    let selection_result = searchable_menu(" GENERAZIONE ASSET", numbered_labels.clone())
        .with_theme(&get_theme())
        .with_page_size(10)
        .with_help_message("Usa i numeri come shortcut • Esc per uscire")
        .prompt();

    selection_result.map_or(GeneratorsActions::Exit, |choice| {
        // Search indice della stringa selezionata nella lista numerata
        let index = numbered_labels
            .iter()
            .position(|s| s == &choice)
            .unwrap_or(items.len() - 1);

        items[index].1
    })
}
