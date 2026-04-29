//!  Fornisce funzioni per eseguire comandi nella shell.
//!
//! Gestisce gli errori e restituisce lo stato di successo/fallimento dei comandi.

use crate::ui::printer::error;
use std::process::Command;

/// Esegue un comando di sistema con argomenti opzionali e directory di lavoro.
///
///
/// # Parametri
/// - `cmd`: il comando da eseguire (es. `"flutter"`, `"bash"`).
/// - `args`: lista di argomenti per il comando.
/// - `dir`: directory di lavoro opzionale.
///
/// # Return
/// - `true` se il comando è stato eseguito con successo.
/// - `false` in caso di errore.
pub fn run_command(cmd: &str, args: &[&str], dir: Option<&str>) -> bool {
    let mut command: Command = Command::new(cmd);
    command.args(args);

    if let Some(d) = dir {
        command.current_dir(d);
    }

    match command.status() {
        Ok(status) => {
            if !status.success() {
                error(&format!("Il comando {} {} è fallito", cmd, args.join(" ")));
                return false;
            }
            true
        }
        Err(e) => {
            error(&format!("Impossibile eseguire {cmd}:\n{e}"));
            false
        }
    }
}



/// Esegue un comando in un nuovo terminale (solo macOS).
///
/// # Parametri
/// - `command`: stringa del comando da eseguire.
///
/// # Return
/// - `true` se il comando è stato avviato correttamente su macOS.
/// - `false` su sistemi non-macOS oppure in caso di errore.
pub fn run_in_new_terminal(command: &str) -> bool {
    if !cfg!(target_os = "macos") {
        return false;
    }

    run_command(
        "osascript",
        &[
            "-e",
            &format!("tell application \"Terminal\" to do script \"{command}\""),
        ],
        None,
    )
}

/// Run a dart command with `--flavor` and `--path`
///
/// # Parametri
/// - `tool`: the dart tool to be used
/// - `flavor`: the flavor (optional) to be used
/// - `path`: the path argument for the dart run command
pub fn run_dart(tool: &str, flavor: Option<&str>, path: &str) -> bool {
    let mut args: Vec<&str> = vec!["run", tool];
    if let Some(f) = flavor {
        args.push("--flavor");
        args.push(f);
    }
    args.push("--path");
    args.push(path);
    run_command("dart", &args, None)
}