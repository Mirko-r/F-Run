//! Funzioni di visualizzazione dei menù

use std::process::exit;

use crate::{
    config::{features::FeaturesConfig, frunconfig::FrunConfig},
    core::{
        exit_codes::{CONFIGERROR, OK},
        menu::{
            generators_menu::{GeneratorsActions, show_generators_menu},
            main_menu::{MainMenuAction, show_main_menu},
            menus::{show_adv_menu, show_build_menu, show_build_runner_menu},
        },
        settings::open_settings,
        utils::check_dependencies,
    },
    features::{
        build::builder::run_build,
        fastlane::run_fastlane,
        flutter::{clean_project, dart_run_build, dart_run_watch},
        generators::{gen_icons, gen_splash},
        installer::install_shorebird,
        localization::localize::gen_language,
        shorebird::run_shorebird,
    },
    ui::printer::error_and_exit,
};

/// Mostra il menu principale
pub fn main_menu() {
    loop {
        match show_main_menu() {
            MainMenuAction::Advanced => numeric_menu(
                show_adv_menu,
                &[
                    Box::new(open_settings),
                    Box::new(install_shorebird),
                    Box::new(check_dependencies),
                ],
            ),
            MainMenuAction::Clean => clean_project(false),
            MainMenuAction::CleanF => clean_project(true),
            MainMenuAction::Generate => build_generators_menu(),
            MainMenuAction::BuildRunner => numeric_menu(
                show_build_runner_menu,
                &[Box::new(dart_run_build), Box::new(dart_run_watch)],
            ),
            MainMenuAction::ShorebirdPatch => run_shorebird(false),
            MainMenuAction::Build => build_build_menu(),
            MainMenuAction::Exit => {
                exit(OK);
            }
        }
    }
}

/// Helper generico per menu numerici
///
/// # Parametri
/// - `show_menu`: funzione che mostra il menu e restituisce l'indice selezionato
/// - `actions`: slice di funzioni corrispondenti alle voci del menu
///
fn numeric_menu<F>(mut show_menu: F, actions: &[Box<dyn Fn()>])
where
    F: FnMut() -> usize,
{
    loop {
        match show_menu() {
            idx if idx < actions.len() => actions[idx](),
            _ => break,
        }
    }
}

/// Mostra il menu per la build
///
fn build_build_menu() {
    numeric_menu(
        show_build_menu,
        &[
            Box::new(|| run_build(true)),
            Box::new(|| {
                if let Some(cfg) = FrunConfig::get() {
                    let f: &FeaturesConfig = &cfg.features;
                    if f.fastlane {
                        run_fastlane();
                    } else if f.shorebird {
                        run_shorebird(true);
                    } else {
                        run_build(false);
                    }
                } else {
                    error_and_exit(
                        "Configurazione non inizializzata. Riavvia il programma.",
                        CONFIGERROR,
                    );
                }
            }),
        ],
    );
}

/// Mostra il menu per la generazione
fn build_generators_menu() {
    loop {
        match show_generators_menu() {
            GeneratorsActions::Icons => gen_icons(),
            GeneratorsActions::Splash => gen_splash(),
            GeneratorsActions::Language => gen_language(),
            GeneratorsActions::Exit => break,
        }
    }
}
