//! Gestisce la build dei progetti Flutter
//!
//! Supoorta:
//! - Build per Android (APK e `AppBundle`)
//! - Build per iOS (IPA) con upload su App Store
//! - Build multipiattaforma con gestione dei flavor e ambienti (`development` / `production`)
//!
//! Utilizza configurazioni definite in [`crate::config::frunconfig::FrunConfig`]

use crate::{
    config::frunconfig::FrunConfig,
    core::{exit_codes::CONFIGERROR, menu::menus::show_os_menu, runner::run_command},
    features::{
        build::{android::build_for_android, ios::build_for_ios},
        flavors::ask_flavors_and,
    },
    ui::printer::error_and_exit,
};

/// Esegue la build per la piattaforma selezionata.
///
/// Mostra un menu per scegliere il sistema operativo e, se presente, il flavor.
/// Chiama le funzioni interne [`build_for_android`], [`build_for_ios`] o [`build_for_all`].
///
/// # Parametri
/// - `dev`: se `true`, esegue una build di sviluppo, altrimenti produzione.
///
/// # Panics
/// - Termina il programma se la configurazione non è inizializzata.
/// - Termina il programma se non è possibile ottenere il flavor selezionato (solo se i flavors sono abilitati).
/// - Termina il programma se account e password per ios non sono impostati (solo se iOS è abilitato)
pub fn run_build(dev: bool) {
    let environment: &str = if dev { "development" } else { "production" };

    if let Some(cfg) = FrunConfig::get() {
        let os: usize = show_os_menu(cfg.ios.eabled);

        if os == 3 {
            return;
        }

        ask_flavors_and(|flavor| match os {
            0 => build_for_android(environment, flavor),
            1 => {
                if let (Some(account), Some(password)) =
                    (&cfg.ios.app_store_acc, &cfg.ios.app_store_password)
                {
                    if account.trim().is_empty() || password.trim().is_empty() {
                        error_and_exit(
                            "Configurazione account e password per App Store non trovata",
                            CONFIGERROR,
                        );
                    }

                    build_for_ios(environment, flavor, account, password);
                } else {
                    error_and_exit(
                        "Configurazione account e password per App Store non trovata",
                        CONFIGERROR,
                    )
                }
            }
            2 => {
                if let (Some(account), Some(password)) =
                    (&cfg.ios.app_store_acc, &cfg.ios.app_store_password)
                {
                    if account.trim().is_empty() || password.trim().is_empty() {
                        error_and_exit(
                            "Configurazione account e password per App Store non trovata",
                            CONFIGERROR,
                        );
                    }

                    build_for_all(environment, flavor, account, password);
                } else {
                    error_and_exit(
                        "Configurazione account e password per App Store non trovata",
                        CONFIGERROR,
                    )
                }
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

/// Esegue la build multipiattaforma (Android + iOS).
///
/// # Parametri
/// - `env`: L'ambiente della build, ad esempio `"development"` o `"production"`.
/// - `flavor`: Flavor del progetto da usare, se presente.
/// - `account`: Account Apple ID per caricare l'app su App Store (iOS).
/// - `password`: Password o App-specific password per l'upload su App Store.
fn build_for_all(env: &str, flavor: Option<&str>, account: &str, password: &str) {
    build_for_android(env, flavor);
    build_for_ios(env, flavor, account, password);
}

/// Esegue la build per un target specifico.
///
/// # Parametri
/// - `env`: ambiente (`development` / `production`)
/// - `flavor`: flavor selezionato, se presente
/// - `target`: `apk`, `appbundle` o `ipa`
///
/// # Return
/// `true` se la build ha avuto successo, `false` altrimenti.
pub fn build_target(env: &str, flavor: Option<&str>, target: &str) -> bool {
    let mut args: Vec<String> = vec!["build", target]
        .into_iter()
        .map(ToString::to_string)
        .collect();

    if let Some(f) = flavor {
        args.push("--flavor".into());
        args.push(f.into());
        args.push(format!(
            "--dart-define-from-file=environment/{f}/{env}.json"
        ));
        args.push("-t".into());
        args.push(format!("lib/flavors/main_{f}.dart"));
    } else {
        args.push(format!("--dart-define-from-file=environment/{env}.json"));
        args.push("-t".into());
        args.push("lib/main.dart".into());
    }
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
    run_command("flutter", &args_ref, None)
}
