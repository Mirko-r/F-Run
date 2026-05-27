//! Client HTTP condiviso basato su `ureq` sincrono.
//!
//! Espone un client `ureq::Agent` riutilizzabile per
//! tutte le chiamate di rete del progetto (`auto_updater`, `file_copier`,
//! `easy_localization`, ecc.).
//!
//! Tutti i punti del codice che necessitano di HTTP devono passare da qui,
//! senza istanziare client propri.

use regex_lite::Regex;
use std::{sync::OnceLock, time::Duration};
use ureq::Agent;

// Il runtime Tokio è stato rimosso.
// Usiamo solo un Agent di ureq che gestisce il pool di connessioni in modo sincrono.
static CLIENT: OnceLock<Agent> = OnceLock::new();

fn client() -> &'static Agent {
    CLIENT.get_or_init(|| {
        Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(30)))
            .build()
            .into()
    })
}

/// Traduce `text` da `source_lang` a `target_lang` usando Google Translate (endpoint mobile).
///
/// La chiamata avviene tramite il client `ureq` già inizializzato.
/// Il testo tradotto viene estratto dalla risposta HTML con un pattern `regex-lite`.
///
/// # Parametri
/// - `text`: testo sorgente da tradurre.
/// - `source_lang`: codice lingua sorgente (es. `"it"`).
/// - `target_lang`: codice lingua target (es. `"en"`).
///
/// # Return
/// Il testo tradotto oppure `None` se la risposta non contiene il risultato atteso.
///
/// # Panics
/// - Termina il processo se la regex interna non è valida (invariante hardcoded).
pub fn google_translate(text: &str, source_lang: &str, target_lang: &str) -> Option<String> {
    let encoded: String = text
        .chars()
        .flat_map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~') {
                vec![c]
            } else {
                format!("%{:02X}", c as u32).chars().collect()
            }
        })
        .collect();
    let url =
        format!("https://translate.google.com/m?tl={target_lang}&sl={source_lang}&q={encoded}");

    // Cambiamento qui: call().ok() ci dà la Response,
    // poi accediamo al body mutabile e usiamo read_to_string()
    let html = client()
        .get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .call()
        .ok()?
        .body_mut() // Accediamo al Body
        .read_to_string() // Metodo corretto in ureq 3.x per ottenere una String
        .ok()?;

    let pattern = Regex::new(r#"(?s)class="(?:t0|result-container)">(.*?)<"#)
        .expect("Regex traduzione non valida");

    pattern
        .captures(&html)
        .and_then(|caps| caps.get(1))
        .map(|m| decode_html_entities(m.as_str()))
}

/// Decodifica le HTML entities comuni presenti nel testo tradotto restituito da Google Translate.
fn decode_html_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}
