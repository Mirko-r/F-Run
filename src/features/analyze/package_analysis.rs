//! Definisce lo stato aggregato dell'analisi di un pacchetto Android.

use crate::features::analyze::category_state::CategoryState;

/// Contiene le statistiche di tutte le categorie e la dimensione totale del pacchetto.
#[derive(Default)]
pub struct PackageAnalysis {
    pub total_size: f64,
    pub assets: CategoryState,
    pub code: CategoryState,
    pub meta: CategoryState,
    pub dex: CategoryState,
    pub resources: CategoryState,
    pub icons: CategoryState,
    pub other: CategoryState,
}

/// Numero di byte in un megabyte (1 MB = `1_048_576` byte)
pub const BYTES_IN_MB: f64 = 1_048_576.0;

impl PackageAnalysis {
    /// Crea una nuova analisi vuota.
    pub fn new() -> Self {
        Self::default()
    }

    /// Converte la dimensione in mb
    ///
    /// # Return
    /// - dimensione in mb (`f64`)
    pub fn to_mb(&self) -> f64 {
        self.total_size / BYTES_IN_MB
    }
}
