//! Utility mirate ai path degli artifact Android generati dai workflow di build.

/// Restituisce il percorso dell'AAB Android in base al flavor selezionato.
///
/// # Parametri
/// - `flavor`: flavor selezionato, se presente.
///
/// # Return
/// - `String` contenente il pattern del path dell'AAB.
pub fn android_bundle_path(flavor: Option<&str>) -> String {
    flavor.map_or_else(
        || "build/app/outputs/bundle/release/*.aab".to_string(),
        |f| format!("build/app/outputs/bundle/{f}Release/*.aab"),
    )
}