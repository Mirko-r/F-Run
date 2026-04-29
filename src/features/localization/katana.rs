//! Modulo per la generazione delle traduzioni con `katana_localization`
//!
//! Responsabilità: orchestrazione della generazione dei file language.dart e language.localize.dart
//!
//! Questo modulo viene richiamato dal menu generatori se la feature `katana_localization` è abilitata.

use crate::{
    config::frunconfig::FrunConfig,
    core::exit_codes::{CONFIGERROR, GENERICERROR},
    features::flutter::dart_run_build,
    try_run,
    ui::printer::{error, error_and_exit, ok},
};
use std::path::{Path, PathBuf};
use time::{OffsetDateTime, format_description::BorrowedFormatItem, macros::format_description};

/// Genera le traduzioni del progetto Flutter tramite `katana_localization`.
///
/// Aggiorna `language.dart` e `language.localize.dart` se abilitati
/// nella configurazione.
///
/// # Panics
/// - Termina il processo se la configurazione non è inizializzata.
pub fn gen_language_katana() {
    if let Some(cfg) = FrunConfig::get() {
        if let Some(ref dir) = cfg.features.katana.language_path {
            let path: &Path = Path::new(dir);

            if !path.exists() || !path.is_dir() {
                error(&format!(
                    "La directory specificata in 'language_path' ('{}') non esiste o non è valida.",
                    path.display()
                ));
                return;
            }

            let format: &[BorrowedFormatItem<'_>] =
                format_description!("[year][month][day][hour][minute][second]");
            let date_now: String = OffsetDateTime::now_local()
                .unwrap_or_else(|_| OffsetDateTime::now_utc())
                .format(&format)
                .unwrap_or_else(|e| {
                    error_and_exit(
                        &format!("Impossibile ottenere la data corrente: {e}"),
                        GENERICERROR,
                    )
                });
            let lang_dart: PathBuf = path.join("language.dart");
            let localize_dart: PathBuf = path.join("language.localize.dart");
            let tmp_file: PathBuf = path.join("language.localize.tmp");
            let lang_dart_str = lang_dart.to_string_lossy().into_owned();
            let localize_dart_str = localize_dart.to_string_lossy().into_owned();
            let tmp_file_str = tmp_file.to_string_lossy().into_owned();

            try_run!(
                "sed",
                &[
                    "-i.bk",
                    &format!("s/666/{date_now}/g"),
                    lang_dart_str.as_str(),
                ],
                None,
            );

            dart_run_build();

            try_run!(
                "bash",
                &[
                    "-c",
                    &format!(
                        "sed -e 's/#/\\${{/' -e 's/§/}}/g' {} > {}",
                        localize_dart.display(),
                        tmp_file.display()
                    ),
                ],
                None,
            );

            try_run!(
                "mv",
                &[tmp_file_str.as_str(), localize_dart_str.as_str(),],
                None,
            );

            try_run!(
                "mv",
                &[
                    format!("{}.bk", lang_dart.display()).as_str(),
                    lang_dart_str.as_str(),
                ],
                None,
            );

            ok("Traduzioni scaricate");
        } else {
            error("Campo 'language_path' mancante nel file di configurazione.");
        }
    } else {
        error_and_exit(
            "Configurazione non inizializzata. Riavvia il programma.",
            CONFIGERROR,
        );
    }
}
