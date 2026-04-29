//! Contiene la configurazione globale.
//!
//! La configurazione viene letta da un file YAML (`frun.yaml`) e memorizzata
//! nella variabile statica [`CONFIG`] tramite [`once_cell::sync::OnceCell`].
//!
//! Fornisce una struct [`FrunConfig`] che contiene tutte le opzioni configurabili
//! dell'applicazione, come il supporto per Shorebird, gestione icone/splash,
//! flavors, percorsi delle lingue e credenziali per App Store.

use std::sync::{RwLock, RwLockReadGuard};

use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

use crate::{
    config::features::FeaturesConfig, core::exit_codes::CONFIGERROR, ui::printer::error_and_exit,
};

/// Configurazione globale accessibile in tutta l'app.
pub static CONFIG: OnceCell<RwLock<FrunConfig>> = OnceCell::new();

/// Nome del file di configurazione YAML
pub const FRUNCONFIG: &str = "frun.yaml";

/// Struct che rappresenta tutte le impostazioni configurabili dell'app
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrunConfig {
    /// Configurazione delle features
    pub features: FeaturesConfig,
    /// Configurazione relativa ai flavors
    pub flavors: FlavorConfig,
    ///  Configurazione relativa ad iOS
    pub ios: IosConfig,
}

/// Struct che rappresenta le impostazioni dei flavors
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlavorConfig {
    /// Indica se il progetto usa flavors
    pub enabled: bool,
    /// Lista dei flavors disponibili
    pub list: Option<Vec<String>>,
}

/// Struct che rappresenta le impostazioni per ios
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IosConfig {
    /// Indica se il progetto include iOS
    pub eabled: bool,
    /// Account App Store (per upload)
    pub app_store_acc: Option<String>,
    /// Password App Store (per upload)
    pub app_store_password: Option<String>,
}

impl FrunConfig {
    /// Restituisce la configurazione globale se inizializzata
    ///
    /// # Return
    /// - `Some(RwLockReadGuard)` con la configurazione corrente se inizializzata.
    /// - `None` se la configurazione globale non è ancora stata inizializzata.
    ///
    /// # Panics
    /// - Termina il processo se il lock della configurazione non è leggibile.
    pub fn get() -> Option<RwLockReadGuard<'static, Self>> {
        CONFIG.get().map(|lock| {
            lock.read().unwrap_or_else(|e| {
                error_and_exit(
                    &format!("Impossibile leggere la configurazione: {e}"),
                    CONFIGERROR,
                )
            })
        })
    }

    /// Funzione di helper per ottenere se la configurazione ha almeno un generatore attivato
    ///
    /// # Return
    /// `true` se ha attivo almeno un generatore, altrimenti `false`
    pub const fn has_generators(&self) -> bool {
        self.features.flutter_native_splash
            || self.features.icons_launcher
            || self.features.katana.enabled
    }
}
