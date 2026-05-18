//! Utility condivise per la creazione di progress bar CLI.
//!
//! Questo modulo centralizza la configurazione di `indicatif` in modo che
//! i workflow possano riusare la stessa logica senza duplicare template,
//! fallback e dettagli di stile.

use indicatif::{ProgressBar, ProgressStyle};

/// Template di default per una progress bar lineare.
pub const DEFAULT_PROGRESS_TEMPLATE: &str =
    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}";

/// Sequenza di caratteri di default usata per il rendering della barra.
pub const DEFAULT_PROGRESS_CHARS: &str = "##-";

/// Crea una `ProgressBar` configurabile e riusabile nei workflow CLI.
///
/// Se `total_steps` vale `0`, viene automaticamente forzato a `1` per evitare
/// una barra non valida quando il chiamante vuole solo mostrare stato e messaggi.
///
/// # Parametri
/// - `total_steps`: numero totale di step previsti.
/// - `template`: template `indicatif`; se `None` usa `DEFAULT_PROGRESS_TEMPLATE`.
/// - `progress_chars`: caratteri della barra; se `None` usa `DEFAULT_PROGRESS_CHARS`.
///
/// # Return
/// Ritorna una `ProgressBar` pronta all'uso.
pub fn create_progress_bar(
    total_steps: u64,
    template: Option<&str>,
    progress_chars: Option<&str>,
) -> ProgressBar {
    let progress_bar = ProgressBar::new(total_steps.max(1));

    if let Ok(style) = ProgressStyle::with_template(template.unwrap_or(DEFAULT_PROGRESS_TEMPLATE)) {
        progress_bar
            .set_style(style.progress_chars(progress_chars.unwrap_or(DEFAULT_PROGRESS_CHARS)));
    }

    progress_bar
}
