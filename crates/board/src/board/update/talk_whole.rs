//! The conversation on its own, whole, instead of the list of tasks: the key that shows it, and what it does to the board.
use super::Effect;
use crate::board::state::State;

/// The conversation on its own, whole: the pane shows what the agent you talk to has said, as far back as it goes, instead
/// of the list of tasks. Again, and the list is there.
pub fn toggle(s: &mut State, fx: &mut Vec<Effect>) {
    s.chat_whole = !s.chat_whole;
    s.full = false;
    s.panel_top = 0;
    s.chat_at = None;
    fx.push(Effect::Save);
}
