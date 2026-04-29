use std::time::Duration;

use crossterm::{
    event::{poll, read},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use super::error::MenuResult;

/// Guard che ripristina la raw mode anche in uscita anticipata.
pub(super) struct RawModeGuard {
    enabled: bool,
}

impl RawModeGuard {
    pub(super) fn new() -> MenuResult<Self> {
        enable_raw_mode()?;
        Ok(Self { enabled: true })
    }

    pub(super) fn disable(&mut self) -> MenuResult<()> {
        if self.enabled {
            disable_raw_mode()?;
            self.enabled = false;
        }

        Ok(())
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        if self.enabled {
            let _ = disable_raw_mode();
        }
    }
}

/// Svuota eventuali key event rimasti in coda prima di mostrare un nuovo menu.
pub(super) fn drain_pending_input() -> MenuResult<()> {
    while poll(Duration::from_millis(0))? {
        let _ = read()?;
    }

    Ok(())
}
