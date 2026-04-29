//! Contiene le configurazioni relative alle features modulari del programma

use crate::core::pubspec::Pubspec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Struct che rappresenta le features modulari del programma
pub struct FeaturesConfig {
    /// Abilita Fastlane
    pub fastlane: bool,
    /// Abilita Shorebird
    pub shorebird: bool,
    /// Abilita la generazione di icone launcher
    pub icons_launcher: bool,
    /// Abilita la generazione di splash screen nativi Flutter
    pub flutter_native_splash: bool,
    /// Configurazione relativa a katana
    pub katana: KatanaConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Struct che rappresenta la configurazione di katana
pub struct KatanaConfig {
    /// Abilita la gestione delle lingue tramite Katana
    pub enabled: bool,
    /// Percorso della directory contenente i file delle lingue
    pub language_path: Option<String>,
}

impl FeaturesConfig {
    /// Detection automatica delle feature di localizzazione
    pub fn detect_localization() -> KatanaConfig {
        // Katana
        let katana_enabled = Pubspec::has_dependency("katana_localization");
        KatanaConfig {
            enabled: katana_enabled,
            language_path: None,
        }
    }
}
