use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream;
use std::{convert::Infallible, time::Duration};

use crate::runtime::SnapshotUpdate;
use crate::server::ServerState;

fn snapshot_update_event(update: &SnapshotUpdate) -> Event {
    let payload = match serde_json::to_string(update) {
        Ok(payload) => payload,
        Err(_) => "{}".to_string(),
    };

    Event::default().event("snapshot_updated").data(payload)
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
                return Some((Ok(snapshot_update_event(&current)), (receiver, false)));
            }

            match receiver.changed().await {
                Ok(()) => {
                    let update = receiver.borrow().clone();
                    Some((Ok(snapshot_update_event(&update)), (receiver, false)))
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
