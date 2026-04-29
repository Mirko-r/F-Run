//! Utility per controlli diagnostici e operazioni interattive sulle dipendenze.

use crate::try_run;
use std::io::stdin;

/// Esegue un controllo delle dipendenze tramite `flutter pub outdated`.
pub fn check_dependencies() {
    try_run!("flutter", &["pub", "outdated"], None);

    println!("\nPremi invio per continuare...");
    let _ = stdin().read_line(&mut String::new());
}
