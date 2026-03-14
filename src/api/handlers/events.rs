use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream;
use std::{convert::Infallible, time::Duration};

use crate::api::server::ServerState;
use crate::store::memory::Update;

fn data_changed_event(update: &Update) -> Event {
    let payload = match serde_json::to_string(update) {
        Ok(payload) => payload,
        Err(_) => "{}".to_string(),
    };

    Event::default().event("data_changed").data(payload)
}

pub async fn events_stream(
    State(state): State<ServerState>,
) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    let receiver = state.update_notifier.subscribe();
    let events = stream::unfold(
        (receiver, true),
        |(mut receiver, include_current)| async move {
            if include_current {
                let current = receiver.borrow().clone();
                return Some((Ok(data_changed_event(&current)), (receiver, false)));
            }

            match receiver.changed().await {
                Ok(()) => {
                    let update = receiver.borrow().clone();
                    Some((Ok(data_changed_event(&update)), (receiver, false)))
                }
                Err(_) => None,
            }
        },
    );

    Sse::new(events).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keepalive"),
    )
}
