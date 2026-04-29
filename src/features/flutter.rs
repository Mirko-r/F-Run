//! Utility per la gestione di Flutter
//!
//! Include:
//! - Build automatiche con `build_runner`
//! - Pulizia del progetto e gestione dei file temporanei
//!
//! Utilizza configurazioni definite in [`crate::config::frunconfig::FrunConfig`]

use crate::{
    config::frunconfig::FrunConfig,
    core::{
        exit_codes::PATHERROR,
        runner::{run_command, run_in_new_terminal},
    },
    try_run,
    ui::{
        colors::{GREEN, RESET},
        printer::{error_and_exit, ok, warn},
    },
};

use std::{env::current_dir, path::PathBuf};

/// Esegue la build del progetto Flutter usando `build_runner`.
///
/// Utilizza l'opzione `--delete-conflicting-outputs` per rimuovere eventuali
/// conflitti nella generazione dei file.
pub fn dart_run_build() {
    run_command(
        "dart",
        &[
            "run",
            "build_runner",
            "build",
            "--delete-conflicting-outputs",
        ],
        None,
    );
}

/// Esegue `build_runner watch` in un nuovo terminale.
///
/// # Panics
/// - Termina il processo se non è possibile ottenere la cartella corrente.
pub fn dart_run_watch() {
    let curr_path: PathBuf = current_dir().unwrap_or_else(|e| {
        error_and_exit(
            &format!("Impossibile ottenere la cartella corrente: {e}"),
            PATHERROR,
        );
    });

    let cmd = format!(
        "cd {} && dart run build_runner watch --delete-conflicting-outputs",
        curr_path.display()
    );

    // Su macOS proviamo ad aprire un terminale separato; su altri sistemi
    // facciamo fallback avviando `dart` nel processo corrente.
    if !run_in_new_terminal(&cmd) {
        warn("Nuovo terminale non supportato su questa piattaforma: avvio watch nel processo corrente");
        let curr_dir = curr_path.display().to_string();
        run_command(
            "dart",
            &[
                "run",
                "build_runner",
                "watch",
                "--delete-conflicting-outputs",
            ],
            Some(curr_dir.as_str()),
        );
    }
}

/// Pulisce il progetto Flutter.
///
/// Rimuove cartelle temporanee, aggiorna la cache dei pacchetti, pulisce i file
/// generati da Flutter e iOS (se presente), e ricostruisce i file tramite `build_runner`.
///
/// # Parametri
/// - `force`: se `true`, forza la rimozione completa di `.dart_tool`, `.pub-cache`
///   e la pulizia della cartella iOS.
pub fn clean_project(force: bool) {
    println!("{GREEN}🧹 Pulizia del progetto Flutter 🧹{RESET}");

    if force {
        warn("Operazione forzata");
        try_run!("rm", &["-rf", ".dart_tool", ".pub-cache"], None);
        try_run!("flutter", &["pub", "cache", "repair"], None);

        if let Some(cfg) = FrunConfig::get()
            && cfg.ios.eabled
        {
            println!("{GREEN}🍎 Pulizia della cartella iOS... 🍎{RESET}");
            try_run!("rm", &["-rf", "Podfile.lock"], Some("ios"));
            try_run!("pod", &["update", "--repo-update"], Some("ios"));
            try_run!("pod", &["install"], Some("ios"));
        }
    }

    try_run!("flutter", &["clean"], None);
    try_run!("flutter", &["pub", "get"], None);
    dart_run_build();

    ok("Pulizia completata con successo!");
}
