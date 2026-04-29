//! Fornisce utility per installare strumenti esterni opzionali.

use crate::{
    core::{runner::run_command, utils::is_command_available},
    ui::printer::{ok, warn},
};

/// Installa Shorebird sul sistema se non è già presente.
///
/// Verifica se il comando `shorebird` è disponibile; in caso contrario
/// esegue lo script ufficiale di installazione.
pub fn install_shorebird() {
    if is_command_available("shorebird") {
        warn("Shorebird già presente nel sistema");
        return;
    }

    ok("Installo shorebird...");
    run_command(
        "bash",
        &[
            "-c",
            "curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/shorebirdtech/install/main/install.sh | bash",
        ],
        None,
    );
}