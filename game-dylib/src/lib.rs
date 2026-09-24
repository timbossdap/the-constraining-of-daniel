//! Wrapper for hot-reloadable plugin.
use first_proj::{fyrox::plugin::Plugin, Game};

#[no_mangle]
pub fn fyrox_plugin() -> Box<dyn Plugin> {
    Box::new(Game::default())
}
