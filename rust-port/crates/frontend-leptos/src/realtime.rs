#[cfg(target_arch = "wasm32")]
use leptos::task::spawn_local;
use shared::{RealtimeEvent, RealtimeTopic};

#[cfg(target_arch = "wasm32")]
use futures_util::StreamExt;
use futures_util::future::AbortHandle;
#[cfg(target_arch = "wasm32")]
use futures_util::future::abortable;

pub fn connect_realtime_listener<F>(
    conversation_id: Option<u64>,
    topics: Vec<RealtimeTopic>,
    mut on_event: F,
) -> Option<AbortHandle>
where
    F: FnMut(RealtimeEvent) + 'static,
{
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(url) = crate::api::realtime_ws_url(conversation_id, &topics) {
            let (task, handle) = abortable(async move {
                let Ok(mut websocket) = gloo_net::websocket::futures::WebSocket::open(&url) else {
                    return;
                };

                while let Some(message) = websocket.next().await {
                    let Ok(gloo_net::websocket::Message::Text(payload)) = message else {
                        continue;
                    };

                    let Ok(event) = serde_json::from_str::<RealtimeEvent>(&payload) else {
                        continue;
                    };

                    on_event(event);
                }
            });

            spawn_local(async move {
                let _ = task.await;
            });

            return Some(handle);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = conversation_id;
        let _ = topics;
        let _ = &mut on_event;
    }

    None
}
