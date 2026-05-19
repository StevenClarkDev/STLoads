use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
};

use serde_json::json;
use serial_test::serial;

fn spawn_atmp_endpoint(status: u16, max_requests: usize) -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let count = Arc::new(AtomicUsize::new(0));
    let thread_count = count.clone();

    thread::spawn(move || {
        for stream in listener.incoming().take(max_requests) {
            let mut stream = stream.unwrap();
            let mut buffer = [0_u8; 8192];
            let _ = stream.read(&mut buffer);
            thread_count.fetch_add(1, Ordering::SeqCst);
            let reason = if status < 400 { "OK" } else { "ERROR" };
            let response = format!(
                "HTTP/1.1 {status} {reason}\r\ncontent-length: 2\r\nconnection: close\r\n\r\nOK"
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
    });

    (url, count)
}

#[tokio::test]
#[serial(atmp_outbound_db)]
async fn successful_callback_marks_outbound_event_delivered() {
    let Some(pool) = backend::test_support::prepare_pool().await.unwrap() else {
        return;
    };
    let (url, count) = spawn_atmp_endpoint(200, 1);

    let event_id = db::tms::enqueue_atmp_outbound_event(
        &pool,
        db::tms::EnqueueAtmpOutboundEvent {
            tenant_id: "tenant-p6",
            event_type: "listing_published",
            posting_id: None,
            booking_award_id: None,
            target_url: Some(&url),
            payload: json!({"event":"listing_published","load":"ATMP-P6"}),
            correlation_id: Some("corr-p6"),
        },
    )
    .await
    .unwrap();

    let summary = backend::atmp_outbound::process_due_events(
        &pool,
        backend::atmp_outbound::AtmpOutboundOptions {
            default_base_url: Some(url),
            batch_size: 10,
            max_attempts: 3,
        },
    )
    .await
    .unwrap();
    let event = db::tms::find_atmp_outbound_event_by_id(&pool, event_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(summary.delivered, 1);
    assert_eq!(count.load(Ordering::SeqCst), 1);
    assert_eq!(event.status, "delivered");
    assert_eq!(event.attempt_count, 1);
    assert!(event.sent_at.is_some());
}

#[tokio::test]
#[serial(atmp_outbound_db)]
async fn failed_callback_retries_then_dead_letters_after_max_attempts() {
    let Some(pool) = backend::test_support::prepare_pool().await.unwrap() else {
        return;
    };
    let (url, count) = spawn_atmp_endpoint(500, 2);

    let event_id = db::tms::enqueue_atmp_outbound_event(
        &pool,
        db::tms::EnqueueAtmpOutboundEvent {
            tenant_id: "tenant-p6",
            event_type: "sync_error",
            posting_id: None,
            booking_award_id: None,
            target_url: Some(&url),
            payload: json!({"event":"sync_error","load":"ATMP-P6"}),
            correlation_id: Some("corr-p6"),
        },
    )
    .await
    .unwrap();

    let options = backend::atmp_outbound::AtmpOutboundOptions {
        default_base_url: Some(url.clone()),
        batch_size: 10,
        max_attempts: 2,
    };
    let first = backend::atmp_outbound::process_due_events(&pool, options.clone())
        .await
        .unwrap();
    db::tms::force_atmp_outbound_event_due_for_test(&pool, event_id)
        .await
        .unwrap();
    let second = backend::atmp_outbound::process_due_events(&pool, options)
        .await
        .unwrap();
    let event = db::tms::find_atmp_outbound_event_by_id(&pool, event_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(first.failed, 1);
    assert_eq!(second.dead_lettered, 1);
    assert_eq!(count.load(Ordering::SeqCst), 2);
    assert_eq!(event.status, "dead_letter");
    assert_eq!(event.attempt_count, 2);
    assert!(event.last_error.unwrap().contains("500"));
}
