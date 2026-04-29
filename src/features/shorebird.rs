//! Shorebird Release/Patch
//!
//! Gestisce la build e la pubblicazione delle app tramite Shorebird.
//! Fornisce funzioni per selezionare flavor, costruire la versione corretta
//! (release o patch) e caricare le app su App Store o spostare gli artifact Android
//! nella cartella Download.
//!
//! Utilizza configurazioni definite in [`crate::config::frunconfig::FrunConfig`]

use std::sync::RwLockReadGuard;

use crate::{
    config::frunconfig::FrunConfig,
    core::{
        exit_codes::CONFIGERROR,
        menu::menus::show_flavors_menu,
        utils::{android_bundle_path, move_to_downloads},
    },
    features::build::ios::upload_app,
    try_run,
    ui::printer::error_and_exit,
};

/// Esegue la build e il deploy dell’app tramite Shorebird.
///
/// Gestisce la selezione dei flavor, determina le piattaforme da buildare
/// (Android e opzionalmente iOS), e chiama la funzione interna [`shorebird`]
/// con i parametri corretti.
///
/// # Parametri
/// - `release`: se `true` builda in modalità release, altrimenti patch.
///
/// # Panics
/// - Termina il processo se la configurazione non è inizializzata.
/// - Termina il processo se il flavor selezionato non è recuperabile.
pub fn run_shorebird(release: bool) {
    if let Some(cfg) = FrunConfig::get() {
        let flavor: Option<String> = if cfg.flavors.enabled {
            let flavors: Vec<String> = cfg.flavors.list.clone().unwrap_or_default();
            let selection: usize = show_flavors_menu(&flavors);

            if selection == flavors.len() {
                // Selezionato "Indietro"
                return;
            }

            Some(
                flavors
                    .get(selection)
                    .unwrap_or_else(|| {
                        error_and_exit("Impossibile ottenere il flavor selezionato", CONFIGERROR)
                    })
                    .clone(),
            )
        } else {
            None
        };

        shorebird(release, flavor.as_deref(), &cfg);
    } else {
        error_and_exit(
            "Configurazione non inizializzata. Riavvia il programma.",
            CONFIGERROR,
        );
    }
}

/// Funzione interna che esegue la build e pubblicazione per le piattaforme selezionate.
///
/// # Parametri
/// - `release`: se `true` builda in modalità release, altrimenti patch.
/// - `flavor`: flavor selezionato, se presente.
/// - `cfg`: riferimento alla configurazione [`FrunConfig`].
///
/// # Panics
/// - Termina il processo se mancano credenziali per App Store o altre configurazioni essenziali.
fn shorebird(release: bool, flavor: Option<&str>, cfg: &RwLockReadGuard<'static, FrunConfig>) {
    let mut platforms: Vec<&str> = vec!["android"];
    let command: &str = if release { "release" } else { "patch" };

    if cfg.ios.eabled {
        platforms.push("ios");
    }

    let args: Vec<String> = build_shorebird_args(flavor);

    for platform in platforms {
        let mut cmd_args: Vec<&str> = vec![command, platform];
        cmd_args.extend(args.iter().map(|s| &**s));

        try_run!("shorebird", &cmd_args, None);

        if release {
            match platform {
                "ios" => {
                    if let (Some(account), Some(password)) =
                        (&cfg.ios.app_store_acc, &cfg.ios.app_store_password)
                    {
                        if account.trim().is_empty() || password.trim().is_empty() {
                            error_and_exit(
                                "Configurazione account e password per App Store non trovata",
                                CONFIGERROR,
                            )
                        }

                        upload_app(account, password);
                    } else {
                        error_and_exit(
                            "Configurazione account e password per App Store non trovata",
                            CONFIGERROR,
                        )
                    }
                }
                "android" => move_to_downloads(&android_bundle_path(flavor)),
                _ => unreachable!(),
            }
        }
    }
}

/// Esegue la build e la pubblicazione solo per iOS.
///
/// Utilizza Shorebird per creare la build e carica l’app su App Store utilizzando
/// le credenziali fornite.
///
/// # Parametri
/// - `release`: se `true` builda in modalità release, altrimenti patch.
/// - `flavor`: flavor selezionato, se presente.
/// - `account`: Apple ID per l’upload su App Store.
/// - `password`: password o app-specific password per l’upload.
///
/// # Panics
/// - Termina il processo se Shorebird fallisce o se l'upload su App Store non riesce.
pub fn shorebird_ios_only(release: bool, flavor: Option<&str>, account: &str, password: &str) {
    let command: &str = if release { "release" } else { "patch" };

    let args: Vec<String> = build_shorebird_args(flavor);

    let mut cmd_args: Vec<&str> = vec![command, "ios"];
    cmd_args.extend(args.iter().map(|s| &**s));

    try_run!("shorebird", &cmd_args, None);
    upload_app(account, password);
}

/// Esegue la build e la pubblicazione solo per Android.
///
/// Utilizza Shorebird per creare la build e sposta l’AAB risultante nella cartella Download.
///
/// # Parametri
/// - `release`: se `true` builda in modalità release, altrimenti patch.
/// - `flavor`: flavor selezionato, se presente.
///
/// # Panics
/// - Termina il processo se Shorebird fallisce o se non riesce a spostare l'AAB.
pub fn shorebird_android_only(release: bool, flavor: Option<&str>) {
    let command: &str = if release { "release" } else { "patch" };

    let args: Vec<String> = build_shorebird_args(flavor);

    let mut cmd_args: Vec<&str> = vec![command, "android"];
    cmd_args.extend(args.iter().map(|s| &**s));

    try_run!("shorebird", &cmd_args, None);

    move_to_downloads(&android_bundle_path(flavor));
}

///  Costruisce la lista di argomenti da passare al comando `shorebird`
///
/// Genera il percorso del `main.dart` corretto e il file di environment
/// in base al flavor selezionato.
///
/// # Parametri
/// - `flavor`: flavor selezionato, se presente.
///
/// # Return
/// Vettore di stringhe contenente gli argomenti da passare a `shorebird`.
fn build_shorebird_args(flavor: Option<&str>) -> Vec<String> {
    flavor.map_or_else(
        || {
            vec![
                "--target".to_string(),
                "./lib/main.dart".to_string(),
                "--dart-define-from-file=environment/production.json".to_string(),
            ]
        },
        |f| {
            vec![
                "--target".to_string(),
                format!("./lib/flavors/main_{f}.dart"),
                "--flavor".to_string(),
                f.to_string(),
                format!("--dart-define-from-file=environment/{f}/production.json"),
            ]
        },
    )
}
