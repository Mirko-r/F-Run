use std::{
    fmt::Display,
    io::{Write, stdout},
    time::{Duration, Instant},
};

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, poll, read};

use crate::core::menu::{
    menu_theme::MenuTheme,
    searchable_menu::{
        error::{MenuError, MenuResult},
        terminal::{RawModeGuard, drain_pending_input},
    },
};

const DEFAULT_PAGE_SIZE: usize = 7;
const INITIAL_ENTER_DEBOUNCE_MS: u64 = 140;
const DEFAULT_HELP_MESSAGE: &str = "Usa i numeri come shortcut • Esc per uscire";

/// Costruttore del menu ricercabile.
///
/// # Parametri
/// - `message`: testo del prompt da mostrare.
/// - `options`: lista opzioni selezionabili.
///
/// # Return
/// - Un builder `SearchableMenu` per il prompt interattivo.
pub const fn searchable_menu<T>(message: &str, options: Vec<T>) -> SearchableMenu<'_, T>
where
    T: Display,
{
    SearchableMenu::new(message, options)
}

/// Builder del menu selezionabile con filtro incrementale.
pub struct SearchableMenu<'a, T>
where
    T: Display,
{
    pub message: &'a str,
    pub options: Vec<T>,
    pub help_message: Option<&'a str>,
    pub page_size: usize,
    pub theme: Option<MenuTheme<'a>>,
}

impl<'a, T> SearchableMenu<'a, T>
where
    T: Display,
{
    /// Crea una nuova istanza con impostazioni di default del menu.
    const fn new(message: &'a str, options: Vec<T>) -> Self {
        Self {
            message,
            options,
            help_message: Some(DEFAULT_HELP_MESSAGE),
            page_size: DEFAULT_PAGE_SIZE,
            theme: None,
        }
    }

    /// Imposta il messaggio di aiuto del prompt.
    pub const fn with_help_message(mut self, message: &'a str) -> Self {
        self.help_message = Some(message);
        self
    }

    /// Imposta il numero di elementi visibili per pagina.
    pub const fn with_page_size(mut self, page_size: usize) -> Self {
        self.page_size = page_size;
        self
    }

    /// Imposta il tema grafico da usare nel prompt.
    pub const fn with_theme(mut self, theme: &MenuTheme<'a>) -> Self {
        self.theme = Some(*theme);
        self
    }

    /// Avvia il pre-filtro non bloccante con lista opzioni e conferma diretta.
    ///
    /// # Return
    /// - `Ok(T)` con il valore selezionato.
    /// - `Err(MenuError::Canceled)` se l'utente preme `Esc`.
    /// - altri `Err(MenuError::Io(_))` in caso di errori terminale.
    pub fn prompt(mut self) -> MenuResult<T> {
        if self.options.is_empty() {
            return Err(MenuError::InvalidConfiguration(
                "Available options can not be empty",
            ));
        }

        let mut filter = String::new();
        let mut selected_filtered_pos: usize = 0;
        let prompt_opened_at = Instant::now();
        let mut has_user_interacted = false;
        let mut raw_mode = RawModeGuard::new()?;
        drain_pending_input()?;
        let theme = self.theme.unwrap_or_default();
        let mut rendered_lines =
            self.render_prefilter_frame(&mut stdout(), &theme, &filter, selected_filtered_pos)?;

        loop {
            if poll(Duration::from_millis(50))?
                && let Event::Key(key) = read()?
            {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Esc => {
                        SearchableMenu::<T>::clear_prefilter_frame(&mut stdout(), rendered_lines)?;
                        raw_mode.disable()?;
                        return Err(MenuError::Canceled);
                    }
                    KeyCode::Enter => {
                        if !has_user_interacted
                            && prompt_opened_at.elapsed().as_millis()
                                < u128::from(INITIAL_ENTER_DEBOUNCE_MS)
                        {
                            continue;
                        }

                        has_user_interacted = true;
                        let filtered = self.filtered_indexes(&filter);
                        if let Some(selected_index) = filtered.get(selected_filtered_pos) {
                            return self.finalize_selection(
                                &mut stdout(),
                                &theme,
                                &mut raw_mode,
                                rendered_lines,
                                *selected_index,
                            );
                        }
                    }
                    KeyCode::Up => {
                        has_user_interacted = true;
                        let filtered = self.filtered_indexes(&filter);
                        if !filtered.is_empty() {
                            if selected_filtered_pos == 0 {
                                selected_filtered_pos = filtered.len() - 1;
                            } else {
                                selected_filtered_pos = selected_filtered_pos.saturating_sub(1);
                            }
                        }
                    }
                    KeyCode::Down => {
                        has_user_interacted = true;
                        let filtered = self.filtered_indexes(&filter);
                        if !filtered.is_empty() {
                            selected_filtered_pos = (selected_filtered_pos + 1) % filtered.len();
                        }
                    }
                    KeyCode::Backspace => {
                        has_user_interacted = true;
                        filter.pop();
                        selected_filtered_pos = 0;
                    }
                    KeyCode::Char(c)
                        if !key.modifiers.contains(KeyModifiers::CONTROL)
                            && !key.modifiers.contains(KeyModifiers::ALT) =>
                    {
                        has_user_interacted = true;
                        filter.push(c);
                        selected_filtered_pos = 0;
                    }
                    _ => {}
                }

                let filtered = self.filtered_indexes(&filter);
                if selected_filtered_pos >= filtered.len() {
                    selected_filtered_pos = 0;
                }

                if !filter.is_empty() && filtered.len() == 1 {
                    let selected_index = filtered[0];
                    return self.finalize_selection(
                        &mut stdout(),
                        &theme,
                        &mut raw_mode,
                        rendered_lines,
                        selected_index,
                    );
                }

                SearchableMenu::<T>::clear_prefilter_frame(&mut stdout(), rendered_lines)?;
                rendered_lines = self.render_prefilter_frame(
                    &mut stdout(),
                    &theme,
                    &filter,
                    selected_filtered_pos,
                )?;
            }
        }
    }

    /// Chiude la mini-UI, stampa la risposta e ritorna il valore selezionato.
    fn finalize_selection<W: Write>(
        &mut self,
        writer: &mut W,
        theme: &MenuTheme<'_>,
        raw_mode: &mut RawModeGuard,
        rendered_lines: usize,
        selected_index: usize,
    ) -> MenuResult<T> {
        let answer = self.options[selected_index].to_string();
        SearchableMenu::<T>::clear_prefilter_frame(writer, rendered_lines)?;
        raw_mode.disable()?;
        self.render_answer_line(writer, theme, &answer)?;
        Ok(self.options.swap_remove(selected_index))
    }

    /// Restituisce gli indici delle opzioni compatibili col filtro corrente.
    pub(super) fn filtered_indexes(&self, filter: &str) -> Vec<usize> {
        if filter.is_empty() {
            return (0..self.options.len()).collect();
        }

        let filter_lc = filter.to_lowercase();
        self.options
            .iter()
            .enumerate()
            .filter_map(|(idx, opt)| {
                let text = opt.to_string().to_lowercase();
                if text.contains(&filter_lc) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Compone il messaggio di help mostrato sotto il prompt di ricerca.
    pub(super) fn build_help_message(&self, filter: &str) -> String {
        let count = self.filtered_indexes(filter).len();

        match self.help_message {
            Some(help) if filter.is_empty() => help.to_string(),
            Some(help) => format!("{help} • {count} risultati"),
            None if filter.is_empty() => String::new(),
            None => format!("{count} risultati"),
        }
    }
}
