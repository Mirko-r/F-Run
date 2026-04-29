//! Racchiude funzionalità specifiche come build multipiattaforma, Shorebird, scrcpy e ADB.
//!
//! Contiene le funzionalità principali del progetto legate alla gestione
//! dei tool esterni, build dei progetti Flutter e deployment.
//! Fornisce wrapper e utility per automatizzare operazioni comuni.

pub mod analyze;
pub mod build;
pub mod fastlane;
pub mod flavors;
pub mod flutter;
pub mod generators;
pub mod installer;
pub mod shorebird;
pub mod localization;
