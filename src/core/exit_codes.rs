//! Definisce gli exit codes standard utilizzati.
//!
//! Sono rappresentati come interi `i32` e
//! consentono di distinguere rapidamente tra esecuzione riuscita
//! e diversi tipi di errori, facilitando la gestione dei fallimenti.

/// Tipo per gli exit codes
pub type ExitCode = i32;

/// Uscita ok
pub const OK: ExitCode = 0;

/// Errore generico
pub const GENERICERROR: ExitCode = 1;

/// Errore di configurazione e/o configurazione non trovata
pub const CONFIGERROR: ExitCode = 2;

/// Il file specificato non esiste / non è stato trovato
pub const PATHERROR: ExitCode = 3;

/// Impossibile leggere / scrivere il file
pub const IOERROR: ExitCode = 4;

/// Impossibile eseguire il parse
pub const PARSEERROR: ExitCode = 5;

/// Errore legato alla UI
pub const UIERROR: ExitCode = 6;

/// Impossibile eseguire un comando
pub const COMMANDERROR: ExitCode = 7;
