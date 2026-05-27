//! Modulo di rete centralizzato per F-Run.
//!
//! Raccoglie tutte le operazioni HTTP del progetto in un unico punto,
//! usando un runtime Tokio e un `reqwest::Client` condivisi.
//!
//! I moduli interni (`auto_updater`, `file_copier`, `easy_localization`)
//! devono importare da qui invece di creare client o runtime propri.

pub mod client;
