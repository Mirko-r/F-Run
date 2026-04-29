//! Gestisce la build e la pubblicazione delle app tramite Fastlane.
//!
//! Utilizza configurazioni definite in [`crate::config::frunconfig::FrunConfig`]

use std::{path::Path, sync::RwLockReadGuard};

use regex_lite::Regex;

use crate::{
    config::frunconfig::FrunConfig,
    core::{
        exit_codes::CONFIGERROR,
        menu::menus::show_os_menu,
        utils::{replace_in_file, search_any_in_dir},
    },
    features::{
        build::{android::build_appbundle, ios::build_for_ios},
        flavors::{ask_flavors_and, get_app_id_for_flavor},
        shorebird::{shorebird_android_only, shorebird_ios_only},
    },
    try_run,
    ui::printer::error_and_exit,
};

/// Avvia il flusso Fastlane in base al sistema operativo e ai flavors configurati.
///
/// - Mostra il menu per selezionare OS (Android/iOS/Entrambi).
/// - Permette di selezionare un flavor se abilitato.
/// - Per Android, chiama `fastlane_android`.
/// - Per iOS, chiama `build_for_ios` con credenziali fornite.
///
/// # Panics
/// - Termina il programma se la configurazione non è inizializzata.
/// - Termina il programma se non è possibile ottenere il flavor selezionato (solo se i flavors sono abilitati).
/// - Termina il programma se account e password per ios non sono impostati (solo se iOS è abilitato)
pub fn run_fastlane() {
    if let Some(cfg) = FrunConfig::get() {
        let os: usize = show_os_menu(cfg.ios.eabled);

        if os == 3 {
            return;
        }

        ask_flavors_and(|flavor| match os {
            0 => fastlane_android(flavor, &cfg),
            1 => run_ios(flavor, &cfg),
            2 => {
                fastlane_android(flavor, &cfg);
                run_ios(flavor, &cfg);
            }
            _ => (),
        });
    } else {
        error_and_exit(
            "Configurazione non inizializzata. Riavvia il programma.",
            CONFIGERROR,
        );
    }
}

fn fastlane_android(selected_flavor: Option<&str>, cfg: &RwLockReadGuard<'static, FrunConfig>) {
    let app_file: &str = "android/fastlane/Appfile";
    if let Some(flavor) = selected_flavor {
        let flavor_supply: &String = &format!("supply_{flavor}.json");

        // se il flavor ha il file supply fai fastlane altrimenti build
        if search_any_in_dir(Path::new("android"), flavor_supply, None).is_some_and(|p| p.exists())
        {
            if cfg.features.shorebird {
                shorebird_android_only(true, Some(flavor));
            } else {
                build_appbundle("production", Some(flavor));
            }

            set_json_key_file(app_file, flavor_supply);

            if let Some(app_id) = get_app_id_for_flavor(flavor) {
                set_package_name(app_file, &app_id);
            } else {
                error_and_exit("Flavor non esistente o appid non trovato", CONFIGERROR);
            }

            try_run!(
                "fastlane",
                &["upload_to_internal_test", &format!("flavor:{flavor}")],
                Some("./android")
            );
        } else if cfg.features.shorebird {
            shorebird_android_only(true, Some(flavor));
        } else {
            build_appbundle("production", Some(flavor));
        }
    } else {
        if cfg.features.shorebird {
            shorebird_android_only(true, None);
        } else {
            build_appbundle("production", None);
        }
        try_run!("fastlane", &["upload_to_internal_test"], Some("./android"));
    }
}

fn run_ios(selected_flavor: Option<&str>, cfg: &RwLockReadGuard<'static, FrunConfig>) {
    if let (Some(account), Some(password)) = (&cfg.ios.app_store_acc, &cfg.ios.app_store_password) {
        if account.trim().is_empty() || password.trim().is_empty() {
            error_and_exit(
                "Configurazione account e password per App Store non trovata",
                CONFIGERROR,
            );
        }

        if cfg.features.shorebird {
            shorebird_ios_only(true, selected_flavor, account, password);
        } else {
            build_for_ios("production", selected_flavor, account, password);
        }
    } else {
        error_and_exit(
            "Configurazione account e password per App Store non trovata",
            CONFIGERROR,
        )
    }
}

/// Aggiorna il percorso del json key file in `Appfile`.
///
/// # Parametri
/// - `appfile_path`: Percorso del file Appfile.
/// - `new_value`: Nuovo valore per `json_key_file`.
///
/// # Panics
/// - Termina il programma se il file non può essere letto o scritto.
/// - Termina il programma se non trova nessuna regex nel file.
fn set_json_key_file(appfile_path: &str, new_value: &str) {
    let re: Regex =
        Regex::new(r#"json_key_file\("([^"]*)"\)"#).expect("Impossibile creare la Regex");

    replace_in_file(
        appfile_path,
        &re,
        &format!(r#"json_key_file("{new_value}")"#),
    );
}

/// Aggiorna il package name in `Appfile`.
///
/// # Parametri
/// - `appfile_path`: Percorso del file Appfile.
/// - `new_value`: Nuovo valore per `package_name`.
///
/// # Panics
/// - Termina il programma se il file non può essere letto o scritto.
/// - Termina il programma se non trova nessuna regex nel file.
fn set_package_name(appfile_path: &str, new_value: &str) {
    let re: Regex =
        Regex::new(r#"package_name\("([^"]*)"\)"#).expect("Impossibile creare la Regex");

    replace_in_file(
        appfile_path,
        &re,
        &format!(r#"package_name("{new_value}")"#),
    );
}
