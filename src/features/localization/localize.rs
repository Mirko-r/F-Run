//! Orchestratore per la generazione delle traduzioni.
//!
//! Chiama il flusso katana o easy_localization in base alla feature rilevata.

use crate::{
    config::frunconfig::FrunConfig,
    core::exit_codes::CONFIGERROR,
    features::{
        localization::easy_localization::gen_language_easy,
        localization::katana::gen_language_katana,
    },
    ui::printer::error_and_exit,
};

/// Genera le traduzioni del progetto Flutter, scegliendo il sistema corretto.
///
/// Se `katana_localization` è abilitato, usa katana. Altrimenti, se `easy_localization` è abilitato, usa easy_localization.
/// Se nessuno dei due è presente, termina con errore.
pub fn gen_language() {
    if let Some(cfg) = FrunConfig::get() {
        if cfg.features.katana.enabled {
            gen_language_katana();
        } else if cfg.features.easy_localization.enabled {
            gen_language_easy();
        } else {
            error_and_exit(
                "Nessun sistema di localizzazione supportato rilevato (katana_localization o easy_localization)",
                CONFIGERROR,
            );
        }
    } else {
        error_and_exit(
            "Configurazione non inizializzata. Riavvia il programma.",
            CONFIGERROR,
        );
    }
}
