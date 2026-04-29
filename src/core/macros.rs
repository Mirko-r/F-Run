/// Esegue un comando e termina la funzione corrente se il comando fallisce.
///
/// # Parametri
/// - Tutti gli argomenti vengono passati direttamente a [`crate::core::runner::run_command`].
///   1. `cmd`: il comando da eseguire (es. `"flutter"`, `"bash"`).
///   2. `args`: slice di argomenti (`&[&str]`).
///   3. `dir`: directory di lavoro opzionale (`Option<&str>`).
///
/// # Comportamento
/// - Se il comando ritorna `false`, la macro esegue `return` immediato.
/// - Se il comando ritorna `true`, l’esecuzione della funzione continua normalmente.
#[macro_export]
macro_rules! try_run {
    ($($arg:tt)*) => {
        if !$crate::core::runner::run_command($($arg)*) {
            return;
        }
    };
}
