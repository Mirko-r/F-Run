//! Workflow di localizzazione basato su `easy_localization`.
//!
//! Questo modulo gestisce il ciclo completo di sincronizzazione delle traduzioni:
//! scoperta dei file lingua, scansione dei sorgenti Dart per stringhe `fr:`,
//! generazione di chiavi slug, aggiornamento append-only dei JSON, traduzione
//! automatica dalle stringhe italiane e sostituzione dei literal nel codice.

use crate::{
    core::{
        exit_codes, progress::create_progress_bar, runner::run_command,
        utils::filesystem::find_dart_files,
    },
    ui::printer::{error_and_exit, ok, warn},
};
use indicatif::ProgressBar;
use regex_lite::Regex;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs::create_dir_all,
    fs::{File, read_dir, read_to_string, write},
    io::{BufRead, BufReader},
    path::Path,
};
use tokio::runtime::Builder;
use translators::{GoogleTranslator, Translator};

/// Collezione delle occorrenze `fr:` trovate nei file Dart con relativo sorgente.
type FrOccurrences = Vec<(String, String)>;

/// Mappa ordinata `chiave -> testo` usata per i file lingua.
type LangMap = BTreeMap<String, String>;

/// Insieme di tutte le mappe lingua indicizzate per codice lingua.
type AllLangMaps = HashMap<String, LangMap>;

/// Cache delle traduzioni eseguite, indicizzata per testo sorgente, lingua sorgente e lingua target.
type TranslationCache = HashMap<(String, String, String), String>;

/// Valore sorgente usato per generare o tradurre una singola chiave.
#[derive(Clone)]
struct TranslationSource {
    lang: String,
    value: String,
}

/// Normalizza una stringa sorgente italiana rimovendo il prefisso `fr:`.
///
/// # Parametri
/// - `value`: testo estratto dal sorgente Dart o dai JSON.
///
/// # Return
/// Ritorna il testo pulito e trimmato, pronto per diventare contenuto di un file lingua.
fn normalize_it_source(value: &str) -> String {
    value
        .trim()
        .strip_prefix("fr:")
        .unwrap_or(value.trim())
        .trim()
        .to_string()
}

/// Converte un testo libero in una chiave slug stabile per `easy_localization`.
///
/// La funzione normalizza alcuni caratteri accentati, abbassa il testo in minuscolo
/// e usa solo le prime 3 parole per ottenere una chiave camelCase.
///
/// # Parametri
/// - `value`: testo italiano da trasformare in chiave.
///
/// # Return
/// Ritorna una chiave slug ASCII; se il testo è vuoto produce `key`.
fn slugify_key(value: &str) -> String {
    let lowered = value.to_lowercase();
    let mut words = Vec::new();
    let mut current = String::new();

    for ch in lowered.chars() {
        let mapped = match ch {
            'à' | 'á' | 'â' | 'ã' | 'ä' => 'a',
            'è' | 'é' | 'ê' | 'ë' => 'e',
            'ì' | 'í' | 'î' | 'ï' => 'i',
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' => 'o',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            _ => ch,
        };

        if mapped.is_ascii_alphanumeric() {
            current.push(mapped);
        } else if !current.is_empty() {
            words.push(std::mem::take(&mut current));
        }
    }

    if !current.is_empty() {
        words.push(current);
    }

    if words.is_empty() {
        "key".to_string()
    } else {
        let mut out = String::new();
        for (idx, word) in words.iter().take(3).enumerate() {
            if idx == 0 {
                out.push_str(word);
            } else {
                let mut chars = word.chars();
                if let Some(first) = chars.next() {
                    out.push(first.to_ascii_uppercase());
                    out.push_str(chars.as_str());
                }
            }
        }
        out
    }
}

/// Mappa il codice lingua locale verso il formato accettato dal traduttore remoto.
///
/// # Parametri
/// - `lang`: codice lingua trovato nei file JSON o nella configurazione.
///
/// # Return
/// Ritorna il codice lingua compatibile con `translators`, oppure `None` se non esiste un mapping noto.
fn normalize_target_lang(lang: &str) -> Option<&str> {
    match lang.to_ascii_lowercase().as_str() {
        "it" => Some("it"),
        "en" => Some("en"),
        "fr" => Some("fr"),
        "es" => Some("es"),
        "de" => Some("de"),
        "pt" | "pt_br" => Some("pt"),
        "nl" => Some("nl"),
        "sv" => Some("sv"),
        "da" => Some("da"),
        "fi" => Some("fi"),
        "no" | "nb" => Some("no"),
        "pl" => Some("pl"),
        "cs" => Some("cs"),
        "sk" => Some("sk"),
        "sl" => Some("sl"),
        "ro" => Some("ro"),
        "hu" => Some("hu"),
        "el" => Some("el"),
        "tr" => Some("tr"),
        "ru" => Some("ru"),
        "uk" => Some("uk"),
        "ar" => Some("ar"),
        "he" => Some("he"),
        "ja" => Some("ja"),
        "ko" => Some("ko"),
        "zh" | "zh_cn" => Some("zh-CN"),
        "zh_tw" | "zh_hk" => Some("zh-TW"),
        "hi" => Some("hi"),
        "id" => Some("id"),
        "vi" => Some("vi"),
        "th" => Some("th"),
        _ => None,
    }
}

/// Scopre le lingue disponibili leggendo i file `.json` della cartella traduzioni.
///
/// # Parametri
/// - `dir`: directory che contiene i file lingua di `easy_localization`.
///
/// # Return
/// Ritorna l'elenco dei codici lingua derivati dal nome file senza estensione.
fn discover_languages(dir: &Path) -> Vec<String> {
    let mut langs = Vec::new();
    if let Ok(entries) = read_dir(dir) {
        for entry in entries.flatten() {
            if let Some(ext) = entry.path().extension()
                && ext == "json"
                && let Some(fname) = entry.path().file_stem()
            {
                langs.push(fname.to_string_lossy().to_string());
            }
        }
    }

    langs
}

/// Cerca tutte le occorrenze di stringhe `fr:` nei file Dart dell'app Flutter.
///
/// La ricerca supporta stringhe tra apici singoli, doppi e alcuni casi bare.
/// Ogni occorrenza include anche la posizione file:riga per facilitarne il debug.
///
/// # Parametri
/// - `lib_dir`: cartella `lib/` dell'app Flutter da scandire ricorsivamente.
///
/// # Return
/// Ritorna la lista delle stringhe `fr:` trovate con la relativa provenienza.
///
/// # Panics
/// Può terminare con panic solo se una regex hardcoded del modulo diventa invalida.
fn extract_fr_occurrences_from_dart(lib_dir: &Path) -> FrOccurrences {
    let mut dart_files = Vec::new();
    let mut fr_strings = Vec::new();
    let quoted_double_re =
        Regex::new(r#"\"(fr:[^\"]+)\""#).expect("Regex fr: doppio apice non valida");
    let quoted_single_re =
        Regex::new(r#"'(fr:[^']+)'"#).expect("Regex fr: singolo apice non valida");
    let bare_re = Regex::new(r#"fr:[^\s,);\]}]+"#).expect("Regex fr: bare non valida");

    find_dart_files(lib_dir, &mut dart_files);
    for file_path in dart_files {
        if let Ok(file) = File::open(&file_path) {
            let reader = BufReader::new(file);
            for (i, line) in reader.lines().map_while(Result::ok).enumerate() {
                let mut found_on_line = false;

                for caps in quoted_double_re.captures_iter(&line) {
                    if let Some(m) = caps.get(1) {
                        fr_strings.push((
                            m.as_str().to_string(),
                            format!("{}:{}", file_path.display(), i + 1),
                        ));
                        found_on_line = true;
                    }
                }

                for caps in quoted_single_re.captures_iter(&line) {
                    if let Some(m) = caps.get(1) {
                        fr_strings.push((
                            m.as_str().to_string(),
                            format!("{}:{}", file_path.display(), i + 1),
                        ));
                        found_on_line = true;
                    }
                }

                if !found_on_line {
                    for m in bare_re.find_iter(&line) {
                        fr_strings.push((
                            m.as_str().to_string(),
                            format!("{}:{}", file_path.display(), i + 1),
                        ));
                    }
                }
            }
        }
    }

    fr_strings
}

/// Costruisce la mappa italiana generata a partire dalle stringhe `fr:` trovate nei sorgenti.
///
/// Se due testi diversi collidono sulla stessa chiave (basata sulle prime 3 parole),
/// il workflow termina con errore bloccante.
///
/// # Parametri
/// - `fr_strings`: occorrenze raccolte dai file Dart.
///
/// # Return
/// Ritorna una mappa ordinata `chiave -> testo italiano` pronta per il sync dei JSON.
fn build_generated_it_map(fr_strings: FrOccurrences) -> LangMap {
    let mut generated_key_to_it_value = BTreeMap::new();
    let mut key_to_origin = HashMap::new();

    for (raw_text, source) in fr_strings {
        let cleaned = normalize_it_source(&raw_text);
        if cleaned.is_empty() {
            continue;
        }

        let base_key = slugify_key(&cleaned);
        if let Some((existing_text, existing_source)) = key_to_origin.get(&base_key)
            && existing_text != &cleaned
        {
            error_and_exit(
                &format!(
                    "Collisione chiavi localizzazione: '{}' e '{}' condividono la stessa chiave '{}' (prime 3 parole).\nOrigine 1: {}\nOrigine 2: {}",
                    existing_text, cleaned, base_key, existing_source, source
                ),
                exit_codes::PARSEERROR,
            );
        }

        key_to_origin
            .entry(base_key.clone())
            .or_insert_with(|| (cleaned.clone(), source));
        generated_key_to_it_value.entry(base_key).or_insert(cleaned);
    }

    generated_key_to_it_value
}

/// Inverte la mappa generata italiana per facilitare la sostituzione nei file Dart.
///
/// # Parametri
/// - `generated_key_to_it_value`: mappa `chiave -> testo italiano` già normalizzata.
///
/// # Return
/// Ritorna una mappa `testo italiano -> chiave` usata durante il replace nel codice.
fn build_it_to_key_map(generated_key_to_it_value: &LangMap) -> BTreeMap<String, String> {
    let mut it_to_key = BTreeMap::new();
    for (key, value) in generated_key_to_it_value {
        it_to_key
            .entry(value.clone())
            .or_insert_with(|| key.clone());
    }
    it_to_key
}

/// Aggiunge in coda a un file JSON solo le chiavi mancanti senza riscrivere il resto del contenuto.
///
/// Questo approccio preserva il più possibile formattazione e ordine già presenti nel file.
///
/// # Parametri
/// - `file_path`: percorso del file lingua da aggiornare.
/// - `key_to_value`: nuove coppie `chiave -> valore` da appendere se assenti.
///
/// # Return
/// Ritorna il numero di chiavi effettivamente inserite oppure un messaggio di errore descrittivo.
fn append_missing_entries_to_json(
    file_path: &Path,
    key_to_value: &BTreeMap<String, String>,
) -> Result<usize, String> {
    let mut content = read_to_string(file_path).map_err(|e| {
        format!(
            "Impossibile leggere il file lingua {}: {e}",
            file_path.display()
        )
    })?;

    if content.trim().is_empty() {
        content = "{}".to_string();
    }

    let parsed: Value = serde_json::from_str(&content).map_err(|e| {
        format!(
            "Il file lingua {} non è un JSON valido: {e}",
            file_path.display()
        )
    })?;

    let Some(existing_obj) = parsed.as_object() else {
        return Err(format!(
            "Il file lingua {} deve contenere un oggetto JSON",
            file_path.display()
        ));
    };

    let mut to_add = Vec::new();
    for (key, value) in key_to_value {
        if !existing_obj.contains_key(key) {
            to_add.push((key.clone(), value.clone()));
        }
    }

    if to_add.is_empty() {
        return Ok(0);
    }

    let close_idx = content.rfind('}').ok_or_else(|| {
        format!(
            "Formato JSON inatteso in {}: manca la parentesi di chiusura",
            file_path.display()
        )
    })?;

    let mut before_close = content[..close_idx].to_string();
    let after_close = &content[close_idx + 1..];

    if !existing_obj.is_empty() {
        before_close.push(',');
    }

    for (idx, (key, value)) in to_add.iter().enumerate() {
        if idx == 0 {
            before_close.push('\n');
        } else {
            before_close.push_str(",\n");
        }

        let key_json = serde_json::to_string(key)
            .map_err(|e| format!("Errore serializzazione chiave JSON: {e}"))?;
        let value_json = serde_json::to_string(value)
            .map_err(|e| format!("Errore serializzazione valore JSON: {e}"))?;
        before_close.push_str(&format!("  {key_json}: {value_json}"));
    }

    before_close.push('\n');
    let new_content = format!("{before_close}}}{after_close}");

    write(file_path, new_content).map_err(|e| {
        format!(
            "Impossibile scrivere il file lingua {}: {e}",
            file_path.display()
        )
    })?;

    Ok(to_add.len())
}

/// Legge un file lingua JSON convertendolo in una mappa ordinata di sole stringhe.
///
/// I valori non stringa vengono ignorati per mantenere il contratto atteso dal workflow.
///
/// Se il file è vuoto, viene considerato come oggetto JSON vuoto.
///
/// # Parametri
/// - `file_path`: percorso del file lingua da leggere.
///
/// # Return
/// Ritorna una mappa `chiave -> valore` oppure un errore descrittivo se il file non è valido.
fn read_json_string_map(file_path: &Path) -> Result<BTreeMap<String, String>, String> {
    let content = read_to_string(file_path).map_err(|e| {
        format!(
            "Impossibile leggere il file lingua {}: {e}",
            file_path.display()
        )
    })?;

    if content.trim().is_empty() {
        return Ok(BTreeMap::new());
    }

    let parsed: Value = serde_json::from_str(&content).map_err(|e| {
        format!(
            "Il file lingua {} non è un JSON valido: {e}",
            file_path.display()
        )
    })?;

    let Some(obj) = parsed.as_object() else {
        return Err(format!(
            "Il file lingua {} deve contenere un oggetto JSON",
            file_path.display()
        ));
    };

    let mut map = BTreeMap::new();
    for (key, value) in obj {
        if let Some(text) = value.as_str() {
            map.insert(key.clone(), text.to_string());
        }
    }

    Ok(map)
}

/// Inizializza i file lingua vuoti con un oggetto JSON valido.
///
/// Questo permette al codegen di `easy_localization` di funzionare anche quando
/// un file lingua esiste già ma non contiene ancora alcuna chiave.
///
/// # Parametri
/// - `dir`: directory che contiene i file lingua.
/// - `langs`: elenco delle lingue riconosciute.
fn initialize_empty_language_files(dir: &Path, langs: &[String]) {
    for lang in langs {
        let lang_file = dir.join(format!("{lang}.json"));
        let Ok(content) = read_to_string(&lang_file) else {
            continue;
        };

        if content.trim().is_empty()
            && let Err(e) = write(&lang_file, "{}\n")
        {
            warn(&format!(
                "Impossibile inizializzare il file lingua {}: {e}",
                lang_file.display()
            ));
        }
    }
}

/// Carica tutte le mappe lingua disponibili dalla directory delle traduzioni.
///
/// In caso di file non leggibile o non valido, la lingua viene mantenuta con una mappa vuota
/// e il problema viene segnalato tramite warning.
///
/// # Parametri
/// - `dir`: directory che contiene i file lingua.
/// - `langs`: codici lingua da caricare.
///
/// # Return
/// Ritorna tutte le mappe lingua indicizzate per codice lingua.
fn load_all_lang_maps(dir: &Path, langs: &[String]) -> AllLangMaps {
    let mut all_lang_maps = HashMap::new();

    for lang in langs {
        let lang_file = dir.join(format!("{lang}.json"));
        match read_json_string_map(&lang_file) {
            Ok(map) => {
                all_lang_maps.insert(lang.clone(), map);
            }
            Err(message) => {
                warn(&message);
                all_lang_maps.insert(lang.clone(), BTreeMap::new());
            }
        }
    }

    all_lang_maps
}

/// Raccoglie l'unione di tutte le chiavi già presenti nei JSON e di quelle generate dai sorgenti.
///
/// # Parametri
/// - `all_lang_maps`: contenuto corrente di tutti i file lingua.
/// - `generated`: nuove chiavi derivate dalle stringhe `fr:` trovate nel codice.
///
/// # Return
/// Ritorna l'insieme ordinato di tutte le chiavi che il sync deve considerare.
fn collect_all_keys(all_lang_maps: &AllLangMaps, generated: &LangMap) -> BTreeSet<String> {
    let mut all_keys = BTreeSet::new();

    for map in all_lang_maps.values() {
        for key in map.keys() {
            all_keys.insert(key.clone());
        }
    }
    for key in generated.keys() {
        all_keys.insert(key.clone());
    }

    all_keys
}

/// Seleziona la lingua sorgente preferita per il workflow di sincronizzazione.
///
/// La priorità è: `it`, poi la prima lingua che contiene almeno un valore non vuoto,
/// infine il primo file lingua disponibile.
///
/// # Parametri
/// - `langs`: elenco delle lingue scoperte nella cartella traduzioni.
/// - `all_lang_maps`: mappe lingua caricate dai file JSON.
///
/// # Return
/// Ritorna il codice lingua da usare come base del sync.
fn select_source_language(langs: &[String], all_lang_maps: &AllLangMaps) -> String {
    if langs.iter().any(|lang| lang == "it") {
        return "it".to_string();
    }

    if let Some(lang) = langs.iter().find(|lang| {
        all_lang_maps
            .get(*lang)
            .is_some_and(|map| map.values().any(|value| !value.trim().is_empty()))
    }) {
        return lang.clone();
    }

    langs.first().cloned().unwrap_or_else(|| "it".to_string())
}

/// Costruisce la sorgente autorevole per ogni chiave da sincronizzare.
///
/// La priorità è: lingua sorgente selezionata, poi chiavi generate dal codice Dart,
/// infine fallback da altre lingue o dalla chiave stessa quando manca qualsiasi contenuto.
///
/// # Parametri
/// - `all_keys`: insieme completo delle chiavi note.
/// - `all_lang_maps`: mappe lingua già caricate.
/// - `generated`: mappa generata dai sorgenti Dart con testi `fr:`.
/// - `source_lang`: lingua base selezionata per il sync dei file JSON.
///
/// # Return
/// Ritorna la mappa `chiave -> sorgente` usata per traduzioni e sync.
fn build_source_by_key(
    all_keys: &BTreeSet<String>,
    all_lang_maps: &AllLangMaps,
    generated: &LangMap,
    source_lang: &str,
) -> BTreeMap<String, TranslationSource> {
    let mut source_by_key = BTreeMap::new();

    if let Some(source_map) = all_lang_maps.get(source_lang) {
        for (key, value) in source_map {
            if !value.trim().is_empty() {
                source_by_key.insert(
                    key.clone(),
                    TranslationSource {
                        lang: source_lang.to_string(),
                        value: value.clone(),
                    },
                );
            }
        }
    }

    for (key, value) in generated {
        source_by_key
            .entry(key.clone())
            .or_insert_with(|| TranslationSource {
                lang: "it".to_string(),
                value: value.clone(),
            });
    }

    for key in all_keys {
        if !source_by_key.contains_key(key) {
            let fallback =
                langsource_for_key(key, all_lang_maps).unwrap_or_else(|| TranslationSource {
                    lang: source_lang.to_string(),
                    value: key.clone(),
                });
            source_by_key.insert(key.clone(), fallback);
            warn(&format!(
                "Chiave '{}' assente nella lingua sorgente {}: uso fallback per la traduzione.",
                key, source_lang
            ));
        }
    }

    source_by_key
}

/// Cerca una sorgente di fallback per una chiave usando qualsiasi file lingua disponibile.
///
/// # Parametri
/// - `key`: chiave per cui trovare un valore sorgente.
/// - `all_lang_maps`: mappe lingua già caricate.
///
/// # Return
/// Ritorna il primo valore non vuoto trovato con la relativa lingua, oppure `None`.
fn langsource_for_key(key: &str, all_lang_maps: &AllLangMaps) -> Option<TranslationSource> {
    for (lang, lang_map) in all_lang_maps {
        if let Some(value) = lang_map.get(key)
            && !value.trim().is_empty()
        {
            return Some(TranslationSource {
                lang: lang.clone(),
                value: value.clone(),
            });
        }
    }

    None
}

/// Traduce un testo sorgente verso una lingua target con cache e fallback sicuro.
///
/// Se la lingua target coincide con la lingua sorgente, se il mapping non esiste o se il runtime async manca, la funzione
/// restituisce il testo originale. La progress bar, quando presente, viene sempre avanzata.
///
/// # Parametri
/// - `lang`: lingua target del file da sincronizzare.
/// - `source`: testo e lingua sorgente della chiave da tradurre.
/// - `runtime`: runtime Tokio usato per la chiamata async al traduttore.
/// - `google_translator`: client del traduttore remoto.
/// - `translation_cache`: cache locale `(testo, lingua sorgente, lingua target) -> traduzione`.
/// - `key`: chiave logica in fase di traduzione, usata nei messaggi di warning.
/// - `progress_bar`: barra di avanzamento opzionale da incrementare a ogni item processato.
///
/// # Return
/// Ritorna il testo tradotto oppure il fallback sorgente in caso di problemi.
fn translate_value(
    lang: &str,
    source: &TranslationSource,
    runtime: Option<&tokio::runtime::Runtime>,
    google_translator: &GoogleTranslator,
    translation_cache: &mut TranslationCache,
    key: &str,
    progress_bar: Option<&ProgressBar>,
) -> String {
    if lang.eq_ignore_ascii_case(&source.lang) {
        if let Some(pb) = progress_bar {
            pb.inc(1);
        }
        return source.value.clone();
    }

    let Some(target_lang) = normalize_target_lang(lang) else {
        warn(&format!(
            "Lingua {lang} non mappata per translators: uso lingua sorgente {}.",
            source.lang
        ));
        if let Some(pb) = progress_bar {
            pb.inc(1);
        }
        return source.value.clone();
    };

    let Some(source_lang) = normalize_target_lang(&source.lang) else {
        warn(&format!(
            "Lingua sorgente {} non mappata per translators sulla chiave {key}: uso testo originale.",
            source.lang
        ));
        if let Some(pb) = progress_bar {
            pb.inc(1);
        }
        return source.value.clone();
    };

    let cache_key = (
        source.value.clone(),
        source_lang.to_string(),
        target_lang.to_string(),
    );
    if let Some(cached) = translation_cache.get(&cache_key) {
        if let Some(pb) = progress_bar {
            pb.inc(1);
        }
        return cached.clone();
    }

    let Some(rt) = runtime else {
        if let Some(pb) = progress_bar {
            pb.inc(1);
        }
        return source.value.clone();
    };

    match rt.block_on(async {
        google_translator
            .translate_async(&source.value, source_lang, target_lang)
            .await
    }) {
        Ok(value) => {
            translation_cache.insert(cache_key, value.clone());
            if let Some(pb) = progress_bar {
                pb.inc(1);
            }
            value
        }
        Err(e) => {
            warn(&format!(
                "Traduzione fallita da {} a {lang} (chiave {key}): {e}. Uso testo sorgente.",
                source.lang
            ));
            if let Some(pb) = progress_bar {
                pb.inc(1);
            }
            source.value.clone()
        }
    }
}

/// Sincronizza un singolo file lingua aggiungendo solo le chiavi mancanti.
///
/// Ogni chiave assente viene valorizzata partendo dalla sorgente disponibile e, se necessario,
/// passando attraverso il traduttore automatico.
///
/// # Parametri
/// - `dir`: directory delle traduzioni.
/// - `lang`: codice lingua del file da sincronizzare.
/// - `all_keys`: insieme completo delle chiavi che devono esistere.
/// - `source_by_key`: contenuto sorgente per ogni chiave con la relativa lingua base.
/// - `all_lang_maps`: snapshot dei file lingua già caricati.
/// - `runtime`: runtime Tokio per le traduzioni async.
/// - `google_translator`: client del traduttore remoto.
/// - `translation_cache`: cache locale delle traduzioni già eseguite.
/// - `progress_bar`: progress bar opzionale da aggiornare durante il sync.
///
/// # Return
/// Ritorna il numero di chiavi inserite nel file lingua.
fn sync_language_file(
    dir: &Path,
    lang: &str,
    all_keys: &BTreeSet<String>,
    source_by_key: &BTreeMap<String, TranslationSource>,
    all_lang_maps: &AllLangMaps,
    runtime: Option<&tokio::runtime::Runtime>,
    google_translator: &GoogleTranslator,
    translation_cache: &mut TranslationCache,
    progress_bar: Option<&ProgressBar>,
) -> usize {
    let lang_file = dir.join(format!("{lang}.json"));
    let existing_map = all_lang_maps.get(lang).cloned().unwrap_or_default();
    let mut missing_key_to_value = BTreeMap::new();

    for key in all_keys {
        if existing_map.contains_key(key) {
            continue;
        }

        let source = source_by_key
            .get(key)
            .cloned()
            .unwrap_or_else(|| TranslationSource {
                lang: lang.to_string(),
                value: key.clone(),
            });
        let translated_value = translate_value(
            lang,
            &source,
            runtime,
            google_translator,
            translation_cache,
            key,
            progress_bar,
        );

        missing_key_to_value.insert(key.clone(), translated_value);
    }

    match append_missing_entries_to_json(&lang_file, &missing_key_to_value) {
        Ok(inserted) => inserted,
        Err(message) => {
            warn(&message);
            0
        }
    }
}

/// Stima il numero totale di step necessari per sincronizzare tutte le lingue.
///
/// # Parametri
/// - `langs`: lingue che devono essere sincronizzate.
/// - `all_keys`: insieme completo delle chiavi da garantire.
/// - `all_lang_maps`: stato corrente dei file lingua già caricati.
///
/// # Return
/// Ritorna il numero totale di chiavi mancanti su tutte le lingue.
fn estimate_sync_steps(
    langs: &[String],
    all_keys: &BTreeSet<String>,
    all_lang_maps: &AllLangMaps,
) -> u64 {
    let mut steps = 0_u64;

    for lang in langs {
        let existing_map = all_lang_maps.get(lang).cloned().unwrap_or_default();
        let missing = all_keys
            .iter()
            .filter(|key| !existing_map.contains_key(*key))
            .count() as u64;
        steps += missing;
    }

    steps
}

/// Sostituisce nei file Dart le stringhe `fr:` con l'accesso a `LocaleKeys.*.tr()`.
///
/// # Parametri
/// - `lib_dir`: directory `lib/` dell'app Flutter.
/// - `it_to_key`: mappa `testo italiano -> chiave` usata per il replace.
///
/// # Return
/// Ritorna il numero totale di occorrenze sostituite nei file Dart.
fn replace_fr_strings_with_locale_keys(
    lib_dir: &Path,
    it_to_key: &BTreeMap<String, String>,
) -> usize {
    let mut dart_files = Vec::new();
    find_dart_files(lib_dir, &mut dart_files);

    let mut total_replaced = 0_usize;
    for file_path in dart_files {
        let Ok(content) = read_to_string(&file_path) else {
            continue;
        };

        let mut updated = content.clone();
        let mut replaced_in_file = 0_usize;

        for (it_value, key) in it_to_key {
            let double_quoted = format!("\"fr:{it_value}\"");
            let single_quoted = format!("'fr:{it_value}'");
            let replacement = format!("LocaleKeys.{key}.tr()");

            let count_double = updated.matches(&double_quoted).count();
            if count_double > 0 {
                updated = updated.replace(&double_quoted, &replacement);
                replaced_in_file += count_double;
            }

            let count_single = updated.matches(&single_quoted).count();
            if count_single > 0 {
                updated = updated.replace(&single_quoted, &replacement);
                replaced_in_file += count_single;
            }
        }

        if replaced_in_file > 0 && write(&file_path, updated).is_ok() {
            total_replaced += replaced_in_file;
        }
    }

    total_replaced
}

/// Esegue il code generation di `easy_localization` per aggiornare i file generati.
///
/// # Parametri
/// - `translations_dir`: directory sorgente delle traduzioni JSON.
fn generate_locale_files(translations_dir: &Path) {
    let generated_dir = Path::new("lib/generated");
    if let Err(e) = create_dir_all(generated_dir) {
        warn(&format!(
            "Impossibile creare la cartella {}: {e}",
            generated_dir.display()
        ));
        return;
    }

    let Some(source_path) = translations_dir.to_str() else {
        warn("Path traduzioni non valido: impossibile generare i file easy_localization.");
        return;
    };

    let loader_ok = run_command(
        "dart",
        &[
            "run",
            "easy_localization:generate",
            "-S",
            source_path,
            "-O",
            "lib/generated",
            "-f",
            "keys",
            "-o",
            "codegen_loader.g.dart",
        ],
        None,
    );

    let codegen_loader_path = generated_dir.join("codegen_loader.g.dart");
    let files_exist = codegen_loader_path.exists();

    if loader_ok && files_exist {
        ok("Generazione completata: lib/generated/codegen_loader.g.dart");
    } else {
        warn(
            "Generazione easy_localization incompleta: verifica source path e file codegen_loader.g.dart in lib/generated/.",
        );
    }
}

/// Esegue l'intero workflow di sincronizzazione per `easy_localization`.
///
/// Il flusso comprende: discovery lingue, scansione dei sorgenti Dart, generazione chiavi,
/// traduzione automatica delle chiavi mancanti, aggiornamento dei JSON, replace nel codice
/// e generazione finale dei file supportati da `easy_localization`.
///
/// # Panics
/// Può andare in panic solo nei casi già documentati dai helper interni che compilano regex hardcoded.
pub fn gen_language_easy() {
    let dir = Path::new("assets/translations");
    let lib_dir = Path::new("./lib/");
    let langs = discover_languages(dir);

    if langs.is_empty() {
        warn("Nessuna lingua trovata in assets/translations/");
        return;
    }

    ok("Lingue riconosciute:");
    for lang in &langs {
        println!("-> {lang}");
    }

    if !lib_dir.exists() {
        warn("Cartella lib/ non trovata: impossibile cercare stringhe 'fr:' nei sorgenti.");
        return;
    }

    let fr_strings = extract_fr_occurrences_from_dart(lib_dir);
    if fr_strings.is_empty() {
        warn(
            "Nessuna stringa 'fr:' trovata nel codice Dart. Procedo con sync chiavi dai JSON esistenti.",
        );
    } else {
        ok(&format!("Stringhe trovate : {}", fr_strings.len()));
        for (text, source) in &fr_strings {
            println!("-> {text} ({source})");
        }
    }

    let generated_key_to_it_value = build_generated_it_map(fr_strings);
    let it_to_key = build_it_to_key_map(&generated_key_to_it_value);
    initialize_empty_language_files(dir, &langs);
    let all_lang_maps = load_all_lang_maps(dir, &langs);
    let all_keys = collect_all_keys(&all_lang_maps, &generated_key_to_it_value);

    if all_keys.is_empty() {
        warn("Nessuna chiave trovata: genero comunque i file easy_localization vuoti.");
        generate_locale_files(dir);
        return;
    }

    let source_lang = select_source_language(&langs, &all_lang_maps);
    ok(&format!(
        "Lingua sorgente selezionata per il sync: {source_lang}"
    ));

    let source_by_key = build_source_by_key(
        &all_keys,
        &all_lang_maps,
        &generated_key_to_it_value,
        &source_lang,
    );

    let runtime = match Builder::new_multi_thread().enable_all().build() {
        Ok(rt) => Some(rt),
        Err(e) => {
            warn(&format!(
                "Impossibile inizializzare runtime async: {e}. Uso testo sorgente come fallback."
            ));
            None
        }
    };

    let google_translator = GoogleTranslator::default();
    let mut translation_cache: TranslationCache = HashMap::new();
    let total_steps = estimate_sync_steps(&langs, &all_keys, &all_lang_maps);
    let progress_bar = create_progress_bar(total_steps, None, None);
    progress_bar.set_message("Sincronizzazione traduzioni in corso...");

    let mut total_inserted = 0_usize;
    for lang in &langs {
        progress_bar.set_message(format!("Sincronizzazione lingua: {lang}"));
        let inserted_for_lang = sync_language_file(
            dir,
            lang,
            &all_keys,
            &source_by_key,
            &all_lang_maps,
            runtime.as_ref(),
            &google_translator,
            &mut translation_cache,
            Some(&progress_bar),
        );
        total_inserted += inserted_for_lang;
        println!("-> {lang}: +{inserted_for_lang} nuove chiavi");
    }

    progress_bar.finish_with_message("Sincronizzazione completata");

    ok(&format!(
        "Sync completata: {} chiavi totali, {} inserimenti nei JSON.",
        all_keys.len(),
        total_inserted
    ));

    let replaced = replace_fr_strings_with_locale_keys(lib_dir, &it_to_key);
    if replaced > 0 {
        ok(&format!(
            "Sostituzione completata: {} occorrenze convertite in LocaleKeys.*.tr().",
            replaced
        ));
    } else {
        warn("Nessuna stringa fr: da sostituire nei file Dart.");
    }

    if replaced > 0 || total_inserted > 0 {
        generate_locale_files(dir);
    }
}
