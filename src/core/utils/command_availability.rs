//! Utility per verificare la disponibilità di comandi nel sistema host.

use std::process::Command;

/// Verifica se un comando è disponibile nel sistema tramite `which`.
///
/// # Parametri
/// - `cmd`: nome del comando da verificare.
///
/// # Return
/// - `true` se il comando esiste e può essere eseguito.
/// - `false` altrimenti.
pub fn is_command_available(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}