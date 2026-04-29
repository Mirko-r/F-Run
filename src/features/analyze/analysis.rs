//! Gestisce l'analisi dei pacchetti aab/apk

use std::{fs::File, path::PathBuf};

use comfy_table::{Table, presets::UTF8_FULL};
use glob::glob;
use zip::{ZipArchive, read::ZipFile};

use crate::{
    core::{
        exit_codes::{IOERROR, PARSEERROR},
        utils::has_extension,
    },
    features::analyze::{category_state::CategoryState, package_analysis::PackageAnalysis},
    ui::printer::error_and_exit,
};

/// Rappresenta una voce/categoria nell'analisi di un pacchetto Android (APK/AAB).
///
/// Contiene:
/// - `label`: etichetta della categoria (es. "Assets", "Code", "Icons")
/// - `predicate`: funzione che determina se un file appartiene a questa categoria
/// - `counter`: contatore della categoria (`CategoryState`)
pub type PackageCategoryEntry<'a> = (
    &'static str,
    &'a dyn Fn(&str) -> bool,
    &'a mut CategoryState,
);

/// Analizza il pacchetto android (aab/apk) e stampa una tabella informativa
///
/// # Parametri
/// - `pattern`: pattern per trovare il taget ES: `build/app/outputs/bundle/release/*.aab`
pub fn analyze_android_package(pattern: &str) {
    let path: PathBuf = glob(pattern)
        .unwrap_or_else(|e| {
            error_and_exit(&format!("Impossibile efftuare il parse: {e}"), PARSEERROR)
        })
        .find_map(Result::ok)
        .unwrap_or_else(|| error_and_exit(&format!("Nessun file trovato per {pattern}"), IOERROR));

    let file: File = File::open(path)
        .unwrap_or_else(|e| error_and_exit(&format!("Impossibile aprire il file: {e}"), IOERROR));
    let mut archive: ZipArchive<File> = ZipArchive::new(file).unwrap_or_else(|e| {
        error_and_exit(&format!("Impossibile creare l'archivio: {e}"), IOERROR)
    });

    let mut analysis: PackageAnalysis = PackageAnalysis::new();

    let mut categories: Vec<PackageCategoryEntry> = vec![
        (
            "Assets",
            &|n| n.contains("flutter_assets") || n.contains("assets/"),
            &mut analysis.assets,
        ),
        (
            "Engine & Codice nativo",
            &|n| n.contains("lib/") || has_extension(n, &["so", "dill"]) || n.contains("snapshot"),
            &mut analysis.code,
        ),
        (
            "Metadati",
            &|n| n.starts_with("META-INF") || n.starts_with("CERT"),
            &mut analysis.meta,
        ),
        ("Dex", &|n| has_extension(n, &["dex"]), &mut analysis.dex),
        (
            "Risorse",
            &|n| has_extension(n, &["arsc", "xml"]),
            &mut analysis.resources,
        ),
        (
            "Icone",
            &|n| has_extension(n, &["png", "jpg", "jpeg", "webp"]),
            &mut analysis.icons,
        ),
    ];

    for i in 0..archive.len() {
        let file: ZipFile<'_, File> = archive.by_index(i).unwrap_or_else(|e| {
            error_and_exit(&format!("Impossibile leggere nell'archivio: {e}"), IOERROR)
        });

        let size: f64 = file.compressed_size() as f64;
        analysis.total_size += size;
        let name: &str = file.name();

        if !categories.iter_mut().any(|(_, pred, counter)| {
            if pred(name) {
                counter.add(size);
                true
            } else {
                false
            }
        }) {
            analysis.other.add(size);
        }
    }

    let mut table: Table = Table::new();
    table.set_header(vec!["Categoria", "Dimensione (MB)", "%", "Files"]);
    table.load_preset(UTF8_FULL);

    for (label, _, cat) in &categories {
        table.add_row(vec![
            *label,
            &format!("{:.2}", cat.to_mb()),
            &format!("{:.1}%", cat.percent(analysis.total_size)),
            &cat.count.to_string(),
        ]);
    }

    println!(
        "\nDimensione pacchetto: {:.2} MB\n{table}\n",
        analysis.to_mb()
    );
}
