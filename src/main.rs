#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![allow(clippy::struct_excessive_bools)]

//! F-Run è una utility CLI per la gestione e l’automazione dei progetti Flutter.
//!
//! Le principali funzionalità includono:
//! - **Gestione della configurazione**: rilevazione automatica di flavors Android, supporto iOS,
//!   dipendenze Katana, Shorebird, icone e splash screen. Configurazione persistente tramite `frun.yaml`.
//! - **Build e generazione**: esecuzione di `build_runner` (build / watch), generazione di lingue, icone e splash screen.
//! - **Supporto multi-flavor e multi-platform**: build dedicate per Android (APK, AAB) e iOS (IPA), gestione di flavors e upload automatico su App Store Connect.
//! - **Funzionalità avanzate**: integrazione con Shorebird, Fastlane, duplicazione dello schermo su dispositivi Android tramite scrcpy, gestione di comandi di sistema in terminali separati.
//! - **Utility di supporto**: funzioni per rilevare file e cartelle, verificare la disponibilità di comandi di sistema, spostamento file nella cartella Download, macro di esecuzione sicura dei comandi.
//!
//!
//! F-Run punta a rendere il workflow di sviluppo Flutter più rapido, sicuro e automatizzato, riducendo errori manuali e semplificando operazioni ripetitive.
//!
//! # Note Importanti
//! - I file degli **environments** vanno messi in environment/{flavor}/..
//!
//!     Es:
//!
//!         - environment/xxx/production.json (per flavor)
//!         - environment/production.json (senza falvor)
//!
//! - I file di configurazione per **`icons_launcher` e `flutter_native_splash`** vanno messi in assets/{favor}/icons/...
//!   
//!     Es:
//!
//!         - assets/xxx/icons/flutter_native_splash.yaml (per flavor)
//!         - assets/xxx/icons/icons_launcher.yaml (per flavor)
//!         - assets/icons/flutter_native_splash.yaml (senza flavor)
//!         - assets/icons/icons_launcher.yaml (senza flavor)
//!
//! - I file `main.dart` in caso di presenza dei flavor vanno messi in lib/flavors/main_{flavor}.dart
//!     
//!     Es:
//!     
//!         - lib/flavors/main_tca.dart
//!
//! - I file `supply.json` per fastlane in presenza dei flavor vanno messi in android/supply_{flavor}.json
//!     
//!     Es:
//!     
//!         - android/supply_xxx.json

mod config;
mod core;
mod features;
mod ui;

use std::env::args;

use crate::{
    config::frunconfig::FrunConfig,
    core::{
        exit_codes::{CONFIGERROR, PATHERROR},
        pubspec::Pubspec,
    },
    ui::{
        menu_runner::main_menu,
        printer::{banner, error_and_exit, project_info},
    },
};

/// # Panic
/// Termina il programma se non si è in un progetto Flutter
fn main() {
    // Version flag
    if args().any(|a| a == "-v" || a == "--version") {
        banner();
        return;
    }

    if !Pubspec::exists_in_current_dir() {
        error_and_exit("Non sei in un progetto Flutter", PATHERROR);
    }

    FrunConfig::init().unwrap_or_else(|e| error_and_exit(&e.to_string(), CONFIGERROR));

    banner();
    project_info(&Pubspec::read_pubspec());
    main_menu();
}
