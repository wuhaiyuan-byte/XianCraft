use crate::{AppState, ServerMessage};
use std::sync::Arc;

pub fn handle_who(state: &Arc<AppState>) -> ServerMessage {
    let who_output = crate::ui::generate_who_list(state, true);
    ServerMessage::Description {
        payload: who_output,
    }
}
