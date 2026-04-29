use crate::core::menu::{
    menu_theme::{MenuTheme, StyledText, TextStyle},
    searchable_menu::{error::MenuResult, menu::SearchableMenu},
};

use crossterm::{
    cursor::{MoveToColumn, MoveUp},
    queue,
    style::{Attribute, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};

use std::{fmt::Display, io::Write};

impl<T> SearchableMenu<'_, T>
where
    T: Display,
{
    /// Renderizza una mini-frame coerente con il tema condiviso: prompt, opzioni filtrate e help message.
    pub(super) fn render_prefilter_frame<W: Write>(
        &self,
        writer: &mut W,
        theme: &MenuTheme<'_>,
        filter: &str,
        selected_filtered_pos: usize,
    ) -> MenuResult<usize> {
        let prompt_column = self.prompt_column(theme, filter);
        let help_message = self.build_help_message(filter);
        let filtered = self.filtered_indexes(filter);
        let safe_selected = if filtered.is_empty() {
            0
        } else {
            selected_filtered_pos.min(filtered.len() - 1)
        };
        let page_start = if self.page_size == 0 {
            0
        } else {
            (safe_selected / self.page_size) * self.page_size
        };
        let visible_options = filtered
            .iter()
            .enumerate()
            .skip(page_start)
            .take(self.page_size.max(1))
            .collect::<Vec<_>>();

        queue!(writer, MoveToColumn(0), Clear(ClearType::CurrentLine))?;
        Self::write_styled(writer, theme.prompt_prefix)?;
        queue!(writer, Print(" "))?;
        Self::write_with_style(writer, self.message, theme.prompt)?;
        // message è ora pubblico
        queue!(writer, Print(" "))?;
        Self::write_with_style(writer, filter, theme.text_input)?;

        queue!(writer, Print("\n"), MoveToColumn(0))?;

        if visible_options.is_empty() {
            Self::write_styled(writer, theme.unhighlighted_option_prefix)?;
            queue!(writer, Print(" "))?;
            Self::write_with_style(writer, "Nessun risultato", theme.option)?;
            queue!(writer, Print("\n"), MoveToColumn(0))?;
        } else {
            for (relative_pos, (abs_pos, idx)) in visible_options.iter().enumerate() {
                let is_selected = *abs_pos == safe_selected;
                let prefix = if is_selected {
                    theme.highlighted_option_prefix
                } else {
                    theme.unhighlighted_option_prefix
                };

                let option_style = if is_selected {
                    theme.selected_option
                } else {
                    theme.option
                };

                Self::write_styled(writer, prefix)?;
                queue!(writer, Print(" "))?;
                Self::write_with_style(writer, &self.options[**idx].to_string(), option_style)?;

                if relative_pos + 1 == visible_options.len()
                    && page_start + visible_options.len() < filtered.len()
                {
                    queue!(writer, Print("  ..."))?;
                }

                queue!(writer, Print("\n"), MoveToColumn(0))?;
            }
        }

        queue!(writer, Clear(ClearType::CurrentLine))?;
        if !help_message.is_empty() {
            Self::write_with_style(writer, "[", theme.help_message)?;
            Self::write_with_style(writer, &help_message, theme.help_message)?;
            Self::write_with_style(writer, "]", theme.help_message)?;
        }

        let rendered_lines = 1 + visible_options.len().max(1) + 1;
        queue!(
            writer,
            MoveUp(u16::try_from(rendered_lines.saturating_sub(1)).unwrap_or(u16::MAX)),
            MoveToColumn(prompt_column)
        )?;
        writer.flush()?;
        Ok(rendered_lines)
    }

    /// Pulisce la mini-frame del pre-filtro (prompt, opzioni e help).
    pub(super) fn clear_prefilter_frame<W: Write>(
        writer: &mut W,
        rendered_lines: usize,
    ) -> MenuResult<()> {
        if rendered_lines == 0 {
            return Ok(());
        }

        queue!(writer, MoveToColumn(0))?;

        for idx in 0..rendered_lines {
            queue!(writer, Clear(ClearType::CurrentLine))?;
            if idx + 1 < rendered_lines {
                queue!(writer, Print("\n"), MoveToColumn(0))?;
            }
        }

        if rendered_lines > 1 {
            queue!(
                writer,
                MoveUp(u16::try_from(rendered_lines.saturating_sub(1)).unwrap_or(u16::MAX))
            )?;
        }
        queue!(writer, MoveToColumn(0))?;

        writer.flush()?;
        Ok(())
    }

    /// Renderizza la riga di risposta finale del prompt (prefix + prompt + answer).
    pub(super) fn render_answer_line<W: Write>(
        &self,
        writer: &mut W,
        theme: &MenuTheme<'_>,
        answer: &str,
    ) -> MenuResult<()> {
        queue!(writer, MoveToColumn(0), Clear(ClearType::CurrentLine))?;
        Self::write_styled(writer, theme.answered_prompt_prefix)?;
        queue!(writer, Print(" "))?;
        Self::write_with_style(writer, self.message, theme.prompt)?;
        queue!(writer, Print(" "))?;

        if theme.answer_from_new_line {
            queue!(writer, Print("\n"))?;
        }

        Self::write_with_style(writer, answer, theme.answer)?;
        queue!(writer, Print("\n"))?;
        writer.flush()?;
        Ok(())
    }

    /// Restituisce la colonna su cui riposizionare il cursore dopo il rendering.
    fn prompt_column(&self, theme: &MenuTheme<'_>, filter: &str) -> u16 {
        let width = theme.prompt_prefix.content.chars().count()
            + 1
            + self.message.chars().count()
            + 1
            + filter.chars().count();

        u16::try_from(width + 1).unwrap_or(u16::MAX)
    }

    /// Scrive un contenuto con uno stile del tema condiviso.
    fn write_with_style<W: Write>(
        writer: &mut W,
        content: &str,
        style: TextStyle,
    ) -> MenuResult<()> {
        Self::apply_style(writer, style)?;
        queue!(
            writer,
            Print(content),
            SetAttribute(Attribute::Reset),
            ResetColor
        )?;
        Ok(())
    }

    /// Scrive un token stilizzato preservandone contenuto e stile.
    fn write_styled<W: Write>(writer: &mut W, styled: StyledText<'_>) -> MenuResult<()> {
        Self::write_with_style(writer, styled.content, styled.style)
    }

    /// Applica colori e attributi al writer crossterm corrente.
    fn apply_style<W: Write>(writer: &mut W, style: TextStyle) -> MenuResult<()> {
        if let Some(fg) = style.fg {
            queue!(writer, SetForegroundColor(fg))?;
        }

        if let Some(bg) = style.bg {
            queue!(writer, SetBackgroundColor(bg))?;
        }

        if style.bold {
            queue!(writer, SetAttribute(Attribute::Bold))?;
        }

        if style.italic {
            queue!(writer, SetAttribute(Attribute::Italic))?;
        }

        Ok(())
    }
}
