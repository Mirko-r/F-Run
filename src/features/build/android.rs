//! Gestisce la build dei progetti Flutter per Android

use crate::{
    core::{
        menu::menus::show_android_target_menu,
        utils::{android_bundle_path, move_to_downloads},
    },
    features::{analyze::analysis::analyze_android_package, build::builder::build_target},
    ui::printer::ok,
};

/// Esegue la build Android e sposta gli artifacts nella cartella Downloads.
///
/// Mostra un menu per scegliere il tipo di build (apk/aab/entrambi).
/// Chiama le funzioni interne [`build_apk`], [`build_appbundle`].
///
/// # Parametri
/// - `env`: ambiente (`development` / `production`)
/// - `flavor`: flavor selezionato, se presente
pub fn build_for_android(env: &str, flavor: Option<&str>) {
    let target: usize = show_android_target_menu();

    if target == 3 {
        return;
    }

    match target {
        0 => {
            if build_apk(env, flavor) {
                ok("Trovi il bundle nella tua cartella dei download");
            }
        }
        1 => {
            if build_appbundle(env, flavor) {
                ok("Trovi il bundle nella tua cartella dei download");
            }
        }
        2 => {
            if build_apk(env, flavor) && build_appbundle(env, flavor) {
                ok("Trovi i bundle nella tua cartella dei download");
            }
        }
        _ => unreachable!(),
    }
}

/// Esegue la build dell'apk per android
///
/// # Parametri
/// - `env`: ambiente (`development` / `production`)
/// - `flavor`: flavor selezionato, se presente
///
/// # Return
/// - `true` se il comando è stato eseguito con successo.
/// - `false` in caso di errore.
fn build_apk(env: &str, flavor: Option<&str>) -> bool {
    if build_target(env, flavor, "apk") {
        let path: &str = "build/app/outputs/flutter-apk/*.apk";
        analyze_android_package(path);
        move_to_downloads(path);
        return true;
    }
    false
}

/// Esegue la build dell'aab per android
///
/// # Parametri
/// - `env`: ambiente (`development` / `production`)
/// - `flavor`: flavor selezionato, se presente
///
/// # Return
/// - `true` se il comando è stato eseguito con successo.
/// - `false` in caso di errore.
pub fn build_appbundle(env: &str, flavor: Option<&str>) -> bool {
    if build_target(env, flavor, "appbundle") {
        let bundle_path: String = android_bundle_path(flavor);

        analyze_android_package(&bundle_path);
        move_to_downloads(&bundle_path);

        return true;
    }
    false
}
