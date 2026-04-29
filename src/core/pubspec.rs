//! Fornisce strumenti per leggere e analizzare il file `pubspec.yaml`
//!
//! Permette di verificare l’esistenza del file,
//! leggere i dati principali e controllare la presenza di dipendenze.

use crate::{
    core::exit_codes::{PARSEERROR, PATHERROR},
    ui::printer::error_and_exit,
};
use serde::Deserialize;
use serde_yaml_ng::{Value, from_str};
use std::{collections::HashMap, env::current_dir, fs::read_to_string, path::PathBuf};

/// Nome del pubspec
pub const PUBNAME: &str = "pubspec.yaml";

#[derive(Deserialize, Debug)]
/// Rappresenta il file `pubspec.yaml` di un progetto Flutter.
pub struct Pubspec {
    /// Nome del progetto
    pub name: String,
    /// Descrizione opzionale del progetto
    pub description: Option<String>,
    /// Versione opzionale del progetto
    pub version: Option<String>,
    /// Dipendenze principali
    pub dependencies: Option<HashMap<String, Value>>,
    /// Dipendenze di sviluppo
    #[serde(rename = "dev_dependencies")]
    pub dev_dependencies: Option<HashMap<String, Value>>,
}

impl Pubspec {
    /// Controlla se il file `pubspec.yaml` esiste nella cartella corrente.
    ///
    /// # Return
    /// - `true` se il file esiste.
    /// - `false` altrimenti.
    ///
    /// # Panic
    /// Termina il programma se non è possibile ottenere la cartella corrente
    pub fn exists_in_current_dir() -> bool {
        current_dir()
            .unwrap_or_else(|e| {
                error_and_exit(
                    &format!("Impossibile ottenere la cartella corrente: {e}"),
                    PATHERROR,
                );
            })
            .join(PUBNAME)
            .exists()
    }

    /// Legge e parsifica il file `pubspec.yaml` nella cartella corrente.
    ///
    /// # Return
    /// - [`Pubspec`] con i dati letti.
    ///
    /// # Panic
    /// Termina il programma se il file non esiste o non può essere letto/parsato.
    pub fn read_pubspec() -> Self {
        let pubspec: PathBuf = current_dir()
            .unwrap_or_else(|e| {
                error_and_exit(
                    &format!("Impossibile ottenere la cartella corrente: {e}"),
                    PATHERROR,
                );
            })
            .join(PUBNAME);

        let contents: String = read_to_string(&pubspec).unwrap_or_else(|e| {
            error_and_exit(
                &format!("Impossibile parsare '{}': {}", &pubspec.display(), e),
                PARSEERROR,
            )
        });

        from_str(&contents).unwrap_or_else(|e| {
            error_and_exit(
                &format!("Impossibile parsare '{}': {}", &pubspec.display(), e),
                PARSEERROR,
            )
        })
    }

    /// Verifica se una determinata dipendenza è presente nelle dipendenze principali o di sviluppo.
    ///
    /// # Parametri
    /// - `dep`: nome della dipendenza da cercare.
    ///
    /// # Return
    /// - `true` se la dipendenza esiste.
    /// - `false` altrimenti.
    pub fn has_dependency(dep: &str) -> bool {
        let pubspec: Self = Self::read_pubspec();

        let dep_is_in = |map: &Option<HashMap<String, Value>>| {
            map.as_ref().is_some_and(|d| d.contains_key(dep))
        };

        dep_is_in(&pubspec.dependencies) || dep_is_in(&pubspec.dev_dependencies)
    }
}
