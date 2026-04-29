//! Gestisce la build dei progetti Flutter per iOS e l'upload su App Store

use std::path::PathBuf;

use crate::{
    core::exit_codes::{COMMANDERROR, IOERROR, PARSEERROR, PATHERROR},
    features::build::builder::build_target,
    try_run,
    ui::printer::{error_and_exit, ok},
};

use glob::glob;

/// Esegue la build iOS e carica l'app su App Store Connect.
///
/// # Parametri
/// - `env`: ambiente (`development` / `production`)
/// - `flavor`: flavor selezionato, se presente
/// - `account`: account Apple ID
/// - `password`: password o App-specific password
pub fn build_for_ios(env: &str, flavor: Option<&str>, account: &str, password: &str) {
    if !cfg!(target_os = "macos") {
        error_and_exit(
            "La build iOS richiede macOS (tool Apple: xcrun/altool).",
            COMMANDERROR,
        );
    }

    if build_target(env, flavor, "ipa") {
        upload_app(account, password);
    }
}

/// Carica un file IPA su App Store Connect.
///
/// # Parametri
/// - `account`: account Apple ID
/// - `password`: password o App-specific password
pub fn upload_app(account: &str, password: &str) {
    if !cfg!(target_os = "macos") {
        error_and_exit(
            "Upload IPA su App Store Connect richiede macOS (tool Apple: xcrun/altool).",
            COMMANDERROR,
        );
    }

    let ipa: PathBuf = glob("build/ios/ipa/*.ipa")
        .unwrap_or_else(|e| {
            error_and_exit(
                &format!("Errore durante la ricerca del file .ipa: {e}"),
                PARSEERROR,
            )
        })
        .find_map(Result::ok)
        .unwrap_or_else(|| error_and_exit("File .ipa non trovato", IOERROR));

    let ipa_path = ipa.to_str().unwrap_or_else(|| {
        error_and_exit(
            "Impossibile convertire il percorso del file IPA in UTF-8",
            PATHERROR,
        )
    });

    let args: [&str; 10] = [
        "altool",
        "--upload-app",
        "-f",
        ipa_path,
        "-u",
        account,
        "-p",
        password,
        "--type",
        "ios",
    ];

    try_run!("xcrun", &args, None);

    ok("📱 Uploaded IPA to App Store Connect 📱");
}
