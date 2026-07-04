mod conflict;
mod install;
mod manifest;
mod uninstall;

pub use conflict::scan_conflicts;
pub use install::{install, list_installed, InstalledPlugin};
pub use uninstall::{is_installed, uninstall};
