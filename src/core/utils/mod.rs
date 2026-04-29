//! Racchiude utility condivise a granularità ridotta, organizzate per responsabilità.

pub mod android_artifacts;
pub mod command_availability;
pub mod dependency_checks;
pub mod filesystem;

pub use android_artifacts::android_bundle_path;
pub use command_availability::is_command_available;
pub use dependency_checks::check_dependencies;
pub use filesystem::{has_extension, move_to_downloads, replace_in_file, search_any_in_dir, search_file_in_dir};