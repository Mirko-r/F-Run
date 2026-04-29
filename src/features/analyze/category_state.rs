//! Definisce lo stato numerico di una singola categoria nell'analisi pacchetti.

use crate::features::analyze::package_analysis::BYTES_IN_MB;

/// Rappresenta le statistiche di una categoria (dimensione totale e numero di file).

#[derive(Default)]
pub struct CategoryState {
    pub size: f64,
    pub count: u64,
}

impl CategoryState {
    /// Aggiunge un file alla categoria.
    ///
    /// # Parametri
    /// - `size`: dimensione del file in byte.
    pub const fn add(&mut self, size: f64) {
        self.size += size;
        self.count += 1;
    }

    /// Converte la dimensione in mb
    ///
    /// # Return
    /// - dimensione in mb (`f64`)
    pub fn to_mb(&self) -> f64 {
        self.size / BYTES_IN_MB
    }

    /// Restituisce la dimensione della categoria in percentuale al totale
    ///
    /// # Parametri
    /// - `total`: dimensione totale su cui effetuare la percentuale
    ///
    /// # Return
    /// - percentuale rispetto al totale  
    pub fn percent(&self, total: f64) -> f64 {
        (self.size / total) * 100.00
    }
}
