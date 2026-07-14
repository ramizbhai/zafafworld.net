use axum::extract::ws::{CloseFrame, Message, WebSocket};

fn test_frame() -> axum::extract::ws::CloseFrame<'static> {
    axum::extract::ws::CloseFrame {
        code: 4401,
        reason: std::borrow::Cow::Borrowed("Unauthorized"),
    }
}
