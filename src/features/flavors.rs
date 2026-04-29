//! Contiene funnziondi ed utilità per la gestione dei flavors

use std::{env::current_dir, fs::read_to_string, path::PathBuf};

use regex_lite::Regex;

use crate::{
    config::frunconfig::FrunConfig,
    core::{
        exit_codes::{CONFIGERROR, PATHERROR},
        menu::menus::show_flavors_menu,
    },
    ui::printer::{error, error_and_exit},
};

/// Rileva tutti i flavors Android presenti nel progetto corrente
/// analizzando il file `android/app/build.gradle`.
///
/// # Return
/// - `Vec<String>` con i nomi dei flavors trovati
/// - Vettore vuoto se non ci sono flavors o se il file `build.gradle` non esiste
///
/// # Panics
/// - Termina il processo se non è possibile ottenere la cartella corrente.
/// - Termina il programma se non è possibile creare la regex per la ricerca dei flavors
pub fn get_flavors() -> Vec<String> {
    let curr_path: PathBuf = current_dir().unwrap_or_else(|e| {
        error_and_exit(
            &format!("Impossibile ottenere la cartella corrente: {e}"),
            PATHERROR,
        );
    });

    let gradle: PathBuf = curr_path.join("android/app/build.gradle");
    if !gradle.exists() {
        error("File build.gradle in /app/ non trovato");
        return vec![];
    }

    let content: String = read_to_string(gradle).unwrap_or_default();
    let mut flavors: Vec<String> = vec![];
    let mut inside_flavors: bool = false;
    let mut brace_level: usize = 0;

    let flavor_name_re: Regex = Regex::new(r"^\s*(\w+)\s*\{").expect("Impossibile creare la Regex");

    for line in content.lines() {
        let line_trim: &str = line.trim();

        if line_trim.starts_with("productFlavors") {
            inside_flavors = true;
            brace_level = line_trim.matches('{').count(); // inizializza con parentesi dopo productFlavors {
            continue;
        }

        if inside_flavors {
            brace_level += line_trim.matches('{').count();
            brace_level -= line_trim.matches('}').count();

            if brace_level == 0 {
                inside_flavors = false;
                continue;
            }

            if let Some(caps) = flavor_name_re.captures(line) {
                flavors.push(caps[1].to_string());
            }
        }
    }

    flavors
}

/// Ottiene l'app id dal fil app/build.gradle dato un flavor
///
/// # Parametri
/// - `flavor`: il flavor per cui si vuole ottenere l'app id
///
/// # Return
/// - `Some(String)`: l'app id del flavor.
/// - `None se non trova nulla`
pub fn get_app_id_for_flavor(flavor: &str) -> Option<String> {
    let gradle_path: &str = "android/app/build.gradle";
    let content: String = read_to_string(gradle_path).ok()?;

    let mut inside_flavors: bool = false;
    let mut brace_level: usize = 0;
    let mut current_flavor: Option<String> = None;

    let flavor_re: Regex = Regex::new(r"^\s*(\w+)\s*\{").expect("Impossibile creare la Regex");
    let app_id_re: Regex =
        Regex::new(r#"applicationId\s+"([^"]+)""#).expect("Impossibile creare la Regex");

    for line in content.lines() {
        let line_trim: &str = line.trim();

        if line_trim.starts_with("productFlavors") {
            inside_flavors = true;
            brace_level = line_trim.matches('{').count();
            continue;
        }

        if inside_flavors {
            brace_level += line_trim.matches('{').count();
            brace_level -= line_trim.matches('}').count();

            if brace_level == 0 {
                inside_flavors = false;
                current_flavor = None;
                continue;
            }

            // Se incontriamo un nuovo flavor
            if let Some(caps) = flavor_re.captures(line_trim) {
                current_flavor = Some(caps[1].to_string());
                continue;
            }

            // Se siamo nel flavor giusto, cerca applicationId
            if let Some(current) = &current_flavor
                && current == flavor
                && let Some(caps) = app_id_re.captures(line_trim)
            {
                return Some(caps[1].to_string());
            }
        }
    }

    None
}

/// Mostra il menu dei flavors e esegue un'azione per il flavor selezionato
///
/// Se nessun flavor è abilitato, esegue l'azione con `None`.
///
/// # Parametri
/// - `action`: funzione da eseguire per ogni flavor. Riceve `Option<&str>` come parametro,
///   che è `Some(flavor)` se i flavors sono abilitati, altrimenti `None`.
///
/// # Panics
/// - Termina il processo se la configurazione non è inizializzata.
/// - Termina il processo se non è possibile recuperare il flavor selezionato.
pub fn ask_flavors_and<F>(mut action: F)
where
    F: FnMut(Option<&str>),
{
    if let Some(cfg) = FrunConfig::get() {
        if cfg.flavors.enabled {
            let flavors: Vec<String> = cfg.flavors.list.clone().unwrap_or_default();
            let selection: usize = show_flavors_menu(&flavors);

            if selection == flavors.len() {
                return; // Indietro
            }

            let selected: &String = flavors
                .get(selection)
                .unwrap_or_else(|| {
                    error_and_exit("Impossibile ottenere il flavor selezionato", CONFIGERROR)
                });
            action(Some(selected));
        } else {
            action(None);
        }
    } else {
        error_and_exit(
            "Configurazione non inizializzata. Riavvia il programma.",
            CONFIGERROR,
        );
    }
}
