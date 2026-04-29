//! Gestione delle impostazioni
//!
//! Questo modulo fornisce le funzionalità per aprire e modificare la configurazione
//! tramite un'interfaccia testuale basata su [`ratatui`].
//!
//! Include:
//! - Apertura del TUI
//! - Modifica dei valori della configurazione
//! - Salvataggio e ricaricamento della configurazione globale

use std::{
    io::{Error, ErrorKind, Result},
    rc::Rc,
};

use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    restore,
    style::{Color, Modifier, Style},
    widgets::{List, ListItem, Paragraph},
};

use crate::{
    config::frunconfig::FrunConfig,
    core::exit_codes::{CONFIGERROR, UIERROR},
    ui::{
        printer::{error_and_exit, warn},
        tui::draw_base_box,
    },
};

/// Apre il TUI per attivare/disattivare le funzionalità.
///
/// # Panics
/// - Termina il programma se non è possibile ricaricare la configurazione
/// - Termina il programma se la configurazione non è inizializzata
pub fn open_settings() {
    if let Some(cfg_read_guard) = FrunConfig::get() {
        let mut cfg_clone: FrunConfig = cfg_read_guard.clone();
        drop(cfg_read_guard); // libera subito il read lock

        let result: Result<()> = run_settings_tui(&mut cfg_clone);
        restore();

        match result {
            Ok(()) => {
                // Salva il clone modificato
                cfg_clone.save();

                // Ricarica la configurazione globale aggiornata
                FrunConfig::reload().unwrap_or_else(|e| {
                    error_and_exit(
                        &format!("Impossibile ricaricare la configurazione{e}"),
                        CONFIGERROR,
                    )
                });
            }
            Err(e) => {
                warn(&e.to_string());
            }
        }

        return;
    }
    error_and_exit(
        "Configurazione non inizializzata. Riavvia il programma.",
        CONFIGERROR,
    );
}

/// Avvia l'interfaccia TUI per attivare/disattivare le funzionalità.
///
/// # Parametri
/// - `cfg`: `&mut FrunConfig` - Riferimento mutabile alla configurazione da modificare.
///
/// # Return
/// - `Ok(())` se l'interfaccia termina correttamente,
///   altrimenti un errore IO.
///
/// # Panics
/// - Termina il programma se non è possibile disegnare la box di base
/// - Termina il programma se non è possibile fare il parse degli elementi
fn run_settings_tui(cfg: &mut FrunConfig) -> Result<()> {
    // Menu items
    let mut menu_items: [(&str, &mut bool); 4] = [
        ("Abilita FastLane", &mut cfg.features.fastlane),
        ("Abilita Shorebird", &mut cfg.features.shorebird),
        ("Abilita icons_launcher", &mut cfg.features.icons_launcher),
        (
            "Abilita flutter_native_splash",
            &mut cfg.features.flutter_native_splash,
        ),
    ];
    let mut selected_item: usize = 0;
    let buttons: [&str; 2] = ["ANNULLA", "SALVA & ESCI"];
    let mut selected_button: Option<usize> = None;

    loop {
        draw_base_box(
            "Gestione funzionalità",
            u16::try_from(menu_items.len() + 2).expect("Errore nel parsing"),
            |f, chunks| {
                let items: Vec<ListItem> = menu_items
                    .iter()
                    .enumerate()
                    .map(|(i, (name, value))| {
                        let style: Style = if selected_button.is_none() && selected_item == i {
                            Style::default()
                                .fg(Color::White)
                                .bg(Color::Cyan)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::Black)
                        };
                        ListItem::new(format!("[{}] {name}", if **value { "■" } else { " " }))
                            .style(style)
                    })
                    .collect();
                f.render_widget(List::new(items), chunks[0]);

                // Pulsanti centrati
                let btn_layout: Rc<[Rect]> = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(50 - 10),
                        Constraint::Length(20),
                        Constraint::Length(20),
                        Constraint::Percentage(50 - 10),
                    ])
                    .split(chunks[1]);

                for (i, &btn) in buttons.iter().enumerate() {
                    let style = if selected_button == Some(i) {
                        Style::default()
                            .fg(Color::White)
                            .bg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Black).bg(Color::DarkGray)
                    };
                    f.render_widget(
                        Paragraph::new(btn)
                            .style(style)
                            .alignment(Alignment::Center),
                        btn_layout[i + 1],
                    );
                }
            },
        )
        .unwrap_or_else(|e| {
            error_and_exit(
                &format!("Impossibile disegnare la box di base: {e}"),
                UIERROR,
            )
        });

        // Gestione input
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up => {
                    if selected_button.is_none() {
                        selected_item = selected_item.saturating_sub(1);
                    }
                }
                KeyCode::Down => {
                    if selected_button.is_none() && selected_item < menu_items.len() - 1 {
                        selected_item += 1;
                    }
                }
                KeyCode::Tab => {
                    if let Some(idx) = selected_button {
                        // se siamo sui pulsanti, spostiamo al successivo
                        let next: usize =
                            (menu_items.len() + idx + 1) % (menu_items.len() + buttons.len());
                        if next < menu_items.len() {
                            selected_button = None;
                            selected_item = next;
                        } else {
                            selected_button = Some(next - menu_items.len());
                        }
                    } else {
                        // dal menu al primo pulsante
                        selected_button = Some(0);
                    }
                }
                KeyCode::BackTab => {
                    if let Some(idx) = selected_button {
                        if idx == 0 {
                            // dal primo pulsante torniamo all'ultimo menu item
                            selected_button = None;
                            selected_item = menu_items.len() - 1;
                        } else {
                            selected_button = Some(idx - 1);
                        }
                    } else {
                        // dal menu torniamo all'ultimo pulsante
                        selected_button = Some(buttons.len() - 1);
                    }
                }
                KeyCode::Enter => {
                    if let Some(idx) = selected_button {
                        match buttons[idx] {
                            "ANNULLA" => {
                                return Err(Error::new(ErrorKind::Interrupted, "Annullato"));
                            }
                            "SALVA & ESCI" => {
                                cfg.save();
                                return Ok(());
                            }
                            _ => {}
                        }
                    } else {
                        // Toggle voce menu
                        if let Some((_, value)) = menu_items.get_mut(selected_item) {
                            **value = !**value;
                        }
                    }
                }
                KeyCode::Esc => {
                    return Err(Error::new(ErrorKind::Interrupted, "Annullato"));
                }
                _ => {}
            }
        }
    }
}
