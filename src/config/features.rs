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
    /// Configurazione relativa a `easy_localization`
    pub easy_localization: EasyLocalizationConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Struct che rappresenta la configurazione di katana
pub struct KatanaConfig {
    /// Abilita la gestione delle lingue tramite Katana
    pub enabled: bool,
    /// Percorso della directory contenente i file delle lingue
    pub language_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Struct che rappresenta la configurazione di `easy_localization`
pub struct EasyLocalizationConfig {
    /// Abilita la gestione delle lingue tramite `easy_localization`
    pub enabled: bool,
}

impl FeaturesConfig {
    /// Detection automatica delle feature di localizzazione
    pub fn detect_localization() -> (KatanaConfig, EasyLocalizationConfig) {
        // Katana
        let katana_enabled = Pubspec::has_dependency("katana_localization");
        // Easy Localization
        let easy_enabled = Pubspec::has_dependency("easy_localization");
        (
            KatanaConfig {
                enabled: katana_enabled,
                language_path: None,
            },
            EasyLocalizationConfig {
                enabled: easy_enabled,
            },
        )
    }
}
