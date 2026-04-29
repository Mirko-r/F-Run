//! Definisce il tema grafico condiviso dei menu interattivi nativi della CLI.
//!
//! Espone tipi per la personalizzazione di colori, stili e token testuali dei prompt CLI.

use crossterm::style::Color;

/// Rappresenta uno stile testuale minimale (colore, grassetto, corsivo) per i prompt CLI.
#[derive(Clone, Copy, Default)]
pub struct TextStyle {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
}

impl TextStyle {
    /// Crea uno stile vuoto.
    pub const fn new() -> Self {
        Self {
            fg: None,
            bg: None,
            bold: false,
            italic: false,
        }
    }

    /// Imposta il colore del testo.
    pub const fn with_fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    /// Attiva il grassetto.
    pub const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Attiva il corsivo.
    pub const fn italic(mut self) -> Self {
        self.italic = true;
        self
    }
}

/// Token testuale con contenuto e stile associato, usato nei prompt CLI.
#[derive(Clone, Copy)]
pub struct StyledText<'a> {
    pub content: &'a str,
    pub style: TextStyle,
}

impl<'a> StyledText<'a> {
    /// Crea un token testuale senza stile.
    pub const fn new(content: &'a str) -> Self {
        Self {
            content,
            style: TextStyle::new(),
        }
    }

    /// Imposta il colore del testo del token.
    /// Imposta il colore del testo del token.
    pub const fn with_fg(mut self, color: Color) -> Self {
        self.style = self.style.with_fg(color);
        self
    }
}

/// Tema grafico condiviso per i menu testuali della CLI.
///
/// Definisce colori, prefissi, stili e layout per i prompt e le opzioni.
#[derive(Clone, Copy)]
pub struct MenuTheme<'a> {
    pub prompt_prefix: StyledText<'a>,
    pub prompt: TextStyle,
    pub text_input: TextStyle,
    pub highlighted_option_prefix: StyledText<'a>,
    pub selected_option: TextStyle,
    pub unhighlighted_option_prefix: StyledText<'a>,
    pub option: TextStyle,
    pub answered_prompt_prefix: StyledText<'a>,
    pub answer: TextStyle,
    pub help_message: TextStyle,
    pub answer_from_new_line: bool,
}

/// Restituisce la configurazione grafica condivisa per i menu testuali.
///
/// # Return
/// Tema grafico [`MenuTheme`] usato da tutti i menu CLI.
pub const fn get_theme() -> MenuTheme<'static> {
    MenuTheme {
        prompt_prefix: StyledText::new(" ❯").with_fg(Color::Cyan),
        prompt: TextStyle::new().with_fg(Color::White).bold(),
        text_input: TextStyle::new(),
        highlighted_option_prefix: StyledText::new(" ≫ ").with_fg(Color::Blue),
        selected_option: TextStyle::new().with_fg(Color::Blue).bold(),
        unhighlighted_option_prefix: StyledText::new("   "),
        option: TextStyle::new().with_fg(Color::DarkGrey),
        answered_prompt_prefix: StyledText::new(" ✓ ").with_fg(Color::Cyan),
        answer: TextStyle::new().with_fg(Color::Cyan).bold(),
        help_message: TextStyle::new().with_fg(Color::AnsiValue(24)).italic(),
        answer_from_new_line: false,
    }
}

impl Default for MenuTheme<'_> {
    /// Restituisce il tema grafico di default per i menu CLI.
    fn default() -> Self {
        get_theme()
    }
}
