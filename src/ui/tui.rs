//! Contiene funzioni di utilità per disegnare interfacce testuali (TUI) usando [`ratatui`].

use ratatui::{
    Frame, Terminal, init,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use std::{
    io::{Result, Stdout},
    rc::Rc,
};

/// Disegna un box principale nella TUI con titolo, lista di elementi e area per pulsanti.
///
/// # Parametri
/// - `title`: titolo del box principale.
/// - `items_len`: numero di elementi nella lista; usato per determinare l'altezza minima della lista.
/// - `render_callback`: funzione di callback che riceve:
///     - `f`: riferimento al frame su cui disegnare i widget.
///     - `inner_chunks`: slice di `Rect` rappresentanti le aree interne del blocco (es. lista e pulsanti),
///       da usare per renderizzare i contenuti specifici.
///
/// # Return
/// Restituisce `Ok(())` se il rendering va a buon fine, altrimenti un errore IO.
pub fn draw_base_box<F>(title: &str, items_len: u16, mut render_callback: F) -> Result<()>
where
    F: FnMut(&mut Frame, &[Rect]),
{
    let mut terminal: Terminal<CrosstermBackend<Stdout>> = init();
    terminal
        .draw(|f| {
            let size: Rect = f.area();

            // Sfondo completo
            f.render_widget(
                Paragraph::new(" ").style(Style::default().bg(Color::Black)),
                size,
            );

            // Layout verticale: lista + pulsanti
            let chunks: Rc<[Rect]> = Layout::default()
                .direction(Direction::Vertical)
                .margin(10)
                .constraints([Constraint::Min(items_len), Constraint::Length(1)])
                .split(size);

            // Blocco principale
            let block: Block<'_> = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::DarkGray))
                .border_style(Style::default().fg(Color::Black));
            f.render_widget(&block, chunks[0]);

            // Layout verticale dentro il blocco: lista + pulsanti
            let inner_chunks: Rc<[Rect]> = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(items_len), // lista
                    Constraint::Length(1),      // area pulsanti
                ])
                .split(block.inner(chunks[0]));

            render_callback(f, &inner_chunks);
        })
        .map(|_| ())
}
