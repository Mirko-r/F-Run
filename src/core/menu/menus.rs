/// Funzioni di orchestrazione per la creazione di menu interattivi CLI.
///
/// Usa il menu nativo del progetto con filtro incrementale e shortcut numeriche.
/// 
use std::string::String;

use crate::core::menu::{menu_theme::get_theme, searchable_menu::menu::searchable_menu};

/// Mostra il menu per le azioni di build runner.
///
/// # Return
/// Indice dell'elemento selezionato.
pub fn show_build_runner_menu() -> usize {
    build_menu(&["Build", "Watch", "Indietro"], "Seleziona un azione")
}

/// Mostra il menu per selezionare un flavor.
///
/// # Parametri
/// * `items` - Lista dei flavor disponibili.
///
/// # Return
/// Indice dell'elemento selezionato.
pub fn show_flavors_menu(items: &[String]) -> usize {
    let mut item_refs: Vec<&str> = items.iter().map(String::as_str).collect();
    item_refs.push("Indietro");

    build_menu(item_refs.as_ref(), "Seleziona un flavor")
}

/// Mostra il menu delle azioni avanzate.
///
/// # Return
/// Indice dell'elemento selezionato.
pub fn show_adv_menu() -> usize {
    build_menu(
        &[
            "Gestione funzionalità",
            "Installa shorebird",
            "Controllo dipendenze",
            "Indietro",
        ],
        "Seleziona un azione",
    )
}

/// Mostra il menu per selezionare la piattaforma di destinazione.
///
/// # Parametri
/// * `ios` - `true` se il progetto supporta iOS.
///
/// # Return
/// Se `ios` è `false`, ritorna direttamente `0` (solo Android), altrimenti indice selezionato.
pub fn show_os_menu(ios: bool) -> usize {
    // Se ha solo Android ritorna direttamente
    if !ios {
        return 0;
    }

    build_menu(
        &["Android", "iOS", "Entrambi", "Indietro"],
        "Seleziona un os",
    )
}

/// Mostra il menu per selezionare il tipo di target per android.
///
/// # Return
/// Indice dell'elemento selezionato.
pub fn show_android_target_menu() -> usize {
    build_menu(
        &["apk", "aab", "Entrambi", "Indietro"],
        "Seleziona un target",
    )
}

/// Mostra il menu per selezionare il tipo di build.
///
/// # Return
/// Indice dell'elemento selezionato.
pub fn show_build_menu() -> usize {
    build_menu(
        &["Sviluppo", "Produzione", "Indietro"],
        "Seleziona un azione",
    )
}

/// Crea un menu CLI con estetica Modern e shortcut numeriche.
///
/// # Parametri
/// * `items` - Slice di stringhe da mostrare come opzioni.
/// * `prompt` - Messaggio da mostrare come prompt.
///
/// # Return
/// Indice dell'elemento selezionato.
fn build_menu(items: &[&str], prompt: &str) -> usize {
    // label numerate: "1. foo", "2. bar", ecc.
    let numbered_items: Vec<String> = items
        .iter()
        .enumerate()
        .map(|(i, &name)| format!("{}. {}", i + 1, name))
        .collect();

    let clean_prompt = format!(" {}", prompt.to_uppercase());

    println!();

    // Show menu
    let result = searchable_menu(&clean_prompt, numbered_items.clone())
        .with_theme(&get_theme())
        .with_page_size(10)
        .with_help_message("Usa i numeri come shortcut • Esc per uscire")
        .prompt();

    result.map_or_else(|_| items.len() - 1, |choice| 
        // Search indice della stringa selezionata nella lista numerata
        numbered_items
            .iter()
            .position(|x| x == &choice)
            .unwrap_or(items.len() - 1)
    )
}
