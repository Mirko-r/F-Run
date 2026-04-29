//! Si occupa dell'inizializzazione e della gestione della configurazione.
//!
//! Fornisce funzioni per rilevare automaticamente flavors Android, supporto iOS, Katana e altre dipendenze
//! dal progetto Flutter corrente.
//!
//! Contiene le de implementazioni di [`FrunConfig`]

use crate::{
    config::{
        features::{FeaturesConfig, KatanaConfig},
        frunconfig::{CONFIG, FRUNCONFIG, FlavorConfig, FrunConfig, IosConfig},
    },
    core::{
        exit_codes::{CONFIGERROR, IOERROR, PARSEERROR},
        pubspec::Pubspec,
        utils::{search_any_in_dir, search_file_in_dir},
    },
    features::flavors::get_flavors,
    ui::printer::error_and_exit,
};

use serde_yaml_ng::{from_str, to_string};

use std::{
    env::current_dir,
    fs::{read_to_string, remove_file, write},
    path::Path,
    sync::{RwLock, RwLockWriteGuard},
};

// Config globale

impl Default for FrunConfig {
    /// Genera una configurazione di default rilevando automaticamente
    /// le caratteristiche del progetto corrente.
    ///
    /// - Controlla la presenza di Katana, `flutter_native_splash`, `icons_launcher`
    /// - Determina i flavors Android presenti
    /// - Rileva la presenza di un progetto iOS
    /// - Determina se Shorebird è abilitato
    ///
    /// # Return
    /// - [`FrunConfig`] con i dati di default
    fn default() -> Self {
        // Detection localizzazione
        let katana = FeaturesConfig::detect_localization();

        let katana_language_path: Option<String> = if katana.enabled {
            search_file_in_dir(Path::new("lib"), "language.dart")
                .map(|p| p.to_string_lossy().to_string())
        } else {
            None
        };

        let mut detected_flavors: Vec<String> = Vec::new();
        if Path::new("android/app/src").exists() {
            detected_flavors.extend(get_flavors());
        }

        let has_flavors: bool = !detected_flavors.is_empty();
        let flavors: Option<Vec<String>> = if has_flavors {
            Some(detected_flavors)
        } else {
            None
        };

        let has_ios: bool = search_any_in_dir(Path::new("ios/Runner"), "Info", Some(".plist"))
            .is_some_and(|p| p.exists());

        Self {
            features: FeaturesConfig {
                fastlane: search_any_in_dir(Path::new("android"), "Fastfile", None)
                    .is_some_and(|p| p.exists()),
                shorebird: search_file_in_dir(
                    current_dir()
                        .unwrap_or_else(|e| {
                            error_and_exit(
                                &format!("Impossibile ottenere la cartella corrente: {e}"),
                                IOERROR,
                            )
                        })
                        .as_path(),
                    "shorebird.yaml",
                )
                .is_some_and(|p| p.exists()),
                flutter_native_splash: Pubspec::has_dependency("flutter_native_splash"),
                icons_launcher: Pubspec::has_dependency("icons_launcher"),
                katana: KatanaConfig {
                    enabled: katana.enabled,
                    language_path: katana_language_path,
                },
            },
            flavors: FlavorConfig {
                enabled: has_flavors,
                list: flavors,
            },
            ios: IosConfig {
                eabled: has_ios,
                app_store_acc: if has_ios {
                    Some(String::new())
                } else {
                    None
                },
                app_store_password: if has_ios {
                    Some(String::new())
                } else {
                    None
                },
            },
        }
    }
}

impl FrunConfig {
    /// Inizializza la configurazione globale di F-Run.
    ///
    /// Rileva le caratteristiche del progetto Flutter corrente e salva la configurazione su file.
    ///
    /// # Return
    /// - `Ok(())` se tutto è andato a buon fine
    /// - `Err` se c'è stato un problema durante la scrittura o il parsing del file
    pub fn init() -> Result<(), Box<dyn std::error::Error>> {
        if CONFIG.get().is_some() {
            return Err("Configurazione già inizializzata. Usa reload invece di init.".into());
        }

        // Se esiste un file di configurazione precedente, rimuovilo per evitare conflitti
        if Path::new(FRUNCONFIG).exists() {
            let _ = remove_file(FRUNCONFIG);
        }

        let cfg: Self = {
            let default: Self = Self::default();
            default.save();
            default
        };

        CONFIG
            .set(RwLock::new(cfg))
            .map_err(|_| "Impossibile impostare la configurazione")?;
        Ok(())
    }

    /// Salva la configurazione corrente nel file `frun.yaml`.
    ///
    /// Sovrascrive eventuali modifiche precedenti.
    ///
    /// # Panics
    /// - Termina il programma se non è possibile fare il parse della configurazione
    /// - Termina il programma se non è possibile scrivere la configurazione
    pub fn save(&self) {
        let yaml: String = to_string(self).unwrap_or_else(|e| {
            error_and_exit(
                &format!("Impossibile fare il parse a string: {e}"),
                PARSEERROR,
            )
        });
        write(FRUNCONFIG, yaml).unwrap_or_else(|e| {
            error_and_exit(
                &format!("Impossibile scrivere la configurazione: {e}"),
                IOERROR,
            )
        });
    }

    /// Ricarica la configurazione da file, sovrascrivendo quella in memoria.
    ///
    /// Utile dopo modifiche manuali a `frun.yaml` o script esterni.
    ///
    /// # Panics
    /// - Termina il programma se non è possibile ottenere la configurazione
    /// - Termina il programma se non è possibile ottenere il write lock per la configurazione
    pub fn reload() -> Result<(), Box<dyn std::error::Error>> {
        // Controlla che CONFIG sia inizializzato
        let cfg_lock = CONFIG
            .get()
            .ok_or("CONFIG non inizializzata")
            .unwrap_or_else(|e| {
                error_and_exit(
                    &format!("Impossibile ottenere la configurazione: {e}"),
                    CONFIGERROR,
                )
            });

        // Carica i nuovi dati da file
        let new_cfg = Self::load();

        // Ottieni il write lock e aggiorna il contenuto
        {
            let mut guard: RwLockWriteGuard<'_, Self> = cfg_lock
                .write()
                .map_err(|_| "Impossibile prendere il write lock su CONFIG")?;
            *guard = new_cfg;
        }

        Ok(())
    }

    /// Carica la configurazione dal file `frun.yaml`.
    ///
    ///
    /// # Panics
    /// - Termina il processo se il file di configurazione non esiste.
    /// - Termina il processo se il parsing YAML fallisce.
    pub fn load() -> Self {
        let content = read_to_string(FRUNCONFIG).unwrap_or_else(|e| {
            error_and_exit(&format!("Impossibile leggere {FRUNCONFIG}: {e}"), IOERROR)
        });

        from_str(&content).unwrap_or_else(|e| {
            error_and_exit(
                &format!("Impossibile effettuare il parsing di {FRUNCONFIG}: {e}"),
                PARSEERROR,
            )
        })
    }
}
