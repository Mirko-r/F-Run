//! Utility condivise per ricerca file, manipolazione di path e aggiornamento di contenuti testuali.

use regex_lite::Regex;

use crate::{
    core::{
        exit_codes::{GENERICERROR, IOERROR, PARSEERROR},
        runner::run_command,
    },
    ui::printer::error_and_exit,
};

use std::{
    fs::{read_dir, read_to_string, write},
    path::{Path, PathBuf},
};

/// Cerca un file con nome esatto in una directory e sotto-directory.
///
/// # Parametri
/// - `dir`: directory iniziale in cui cercare.
/// - `file`: nome del file da cercare.
///
/// # Return
/// - `Some(PathBuf)` della directory contenente il file.
/// - `None` se non trovato.
pub fn search_file_in_dir(dir: &Path, file: &str) -> Option<PathBuf> {
    if !dir.is_dir() {
        return None;
    }

    for entry in read_dir(dir).ok()? {
        let path: PathBuf = entry.ok()?.path();

        if path.is_file() && path.file_name().is_some_and(|f| f == file) {
            return path.parent().map(Path::to_path_buf);
        }

        if path.is_dir()
            && let Some(found) = search_file_in_dir(&path, file)
        {
            return Some(found);
        }
    }

    None
}

/// Cerca un file che inizia con un nome specifico e opzionalmente con una estensione.
///
/// # Parametri
/// - `dir`: directory iniziale in cui cercare.
/// - `file`: prefisso del file da cercare.
/// - `extension`: estensione opzionale da confrontare.
///
/// # Return
/// - `Some(PathBuf)` del file trovato.
/// - `None` se non trovato.
pub fn search_any_in_dir(
    dir: &Path,
    file: &str,
    extension: Option<&str>,
) -> Option<PathBuf> {
    for entry in read_dir(dir).ok()? {
        let path: PathBuf = entry.ok()?.path();
        if path.is_file() {
            if let Some(name) = path.file_name()?.to_str() {
                let ext_ok: bool = extension.is_none_or(|ext| name.ends_with(ext));

                if name.starts_with(file) && ext_ok {
                    return Some(path);
                }
            }
        } else if path.is_dir()
            && let Some(found) = search_any_in_dir(&path, file, extension)
        {
            return Some(found);
        }
    }
    None
}

/// Sposta un file o pattern specificato nella cartella `~/Downloads`.
///
/// # Parametri
/// - `path`: percorso del file o pattern da spostare.
pub fn move_to_downloads(path: &str) {
    run_command("bash", &["-c", &format!("mv -f {path} ~/Downloads")], None);
}

/// Verifica se un nome di file termina con una specifica estensione.
///
/// # Parametri
/// - `name`: nome del file da verificare.
/// - `exts`: lista di estensioni da confrontare.
///
/// # Return
/// - `true` se il nome termina con una delle estensioni specificate.
/// - `false` altrimenti.
pub fn has_extension(name: &str, exts: &[&str]) -> bool {
    Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| {
            let ext = ext.to_ascii_lowercase();
            exts.iter().any(|&e| e.eq_ignore_ascii_case(&ext))
        })
}

/// Sostituisce la prima occorrenza di una regex in un file con un nuovo valore.
///
/// # Parametri
/// - `path`: percorso del file in cui effettuare la sostituzione.
/// - `re`: regex da cercare nel file.
/// - `replacement`: stringa di sostituzione.
///
/// # Panics
/// - Termina il processo se il file non può essere letto o scritto.
/// - Termina il processo se la regex non trova alcuna corrispondenza.
pub fn replace_in_file(path: &str, re: &Regex, replacement: &str) {
    let content = read_to_string(path).unwrap_or_else(|e| {
        error_and_exit(
            &format!("Impossibile leggere il file {path} come string: {e}"),
            PARSEERROR,
        )
    });

    let new_content = content.replace(
        re.find(&content)
            .unwrap_or_else(|| {
                error_and_exit(
                    &format!("Impossibile trovare una corrispondenza in {path}"),
                    GENERICERROR,
                )
            })
            .as_str(),
        replacement,
    );
    write(path, new_content).unwrap_or_else(|e| {
        error_and_exit(
            &format!("Impossibile scrivere il file {path}: {e}"),
            IOERROR,
        )
    });
}