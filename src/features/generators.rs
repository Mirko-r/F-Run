//! Funzioni per i generatori
//!
//! - Creazione di icone e splash screen tramite `icons_launcher` e `flutter_native_splash`
//!
//! Utilizza configurazioni definite in [`crate::config::frunconfig::FrunConfig`]

use crate::{core::runner::run_dart, features::flavors::ask_flavors_and};

/// Genera icone per il progetto
///
/// Supporta la generazione per i diversi flavors
///
/// # Panics
/// - Termina il processo se la configurazione non è inizializzata.
pub fn gen_icons() {
    gen_assets(
        "icons_launcher:create",
        "assets/icons/icons_launcher.yaml",
        "assets/{f}/icons/icons_launcher.yaml",
    );
}

/// Genera splash screen per il progetto
///
/// Supporta la generazione per i diversi flavors
///
/// # Panics
/// - Termina il processo se la configurazione non è inizializzata.
pub fn gen_splash() {
    gen_assets(
        "flutter_native_splash:create",
        "assets/icons/flutter_native_splash.yaml",
        "assets/{f}/icons/flutter_native_splash.yaml",
    );
}

/// Genera assets per ogni flavor o per il progetto di default
///
/// # Parametri
/// - `tool`: strumento da eseguire (es. `icons_launcher:create`)
/// - `default_path`: percorso del file di configurazione per il progetto senza flavor
/// - `flavor_path_fmt`: formato del percorso del file di configurazione per i flavor
fn gen_assets(tool: &str, default_path: &str, flavor_path_fmt: &str) {
    ask_flavors_and(|flavor| {
        #[allow(clippy::literal_string_with_formatting_args)]
        let path = flavor.map_or_else(
            || default_path.to_string(),
            |f| flavor_path_fmt.replace("{f}", f),
        );

        run_dart(tool, flavor, &path);
    });
}
