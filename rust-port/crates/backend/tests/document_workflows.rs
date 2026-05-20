use serial_test::serial;

#[tokio::test]
#[serial(document_workflows_db)]
async fn load_document_replacement_preserves_prior_versions_and_blocks_payment_until_approved() {
    let Some(pool) = backend::test_support::prepare_pool().await.unwrap() else {
        return;
    };

    let fixture = backend::test_support::insert_load_fixture(&pool, 10)
        .await
        .unwrap();

    let created = db::dispatch::create_load_document(
        &pool,
        fixture.load_id,
        &db::dispatch::UpsertLoadDocumentParams {
            document_name: "Proof of delivery".into(),
            document_type: "delivery_pod".into(),
            file_path: "local://load-documents/p13/pod-v1.pdf".into(),
            storage_provider: "local".into(),
            original_name: Some("pod-v1.pdf".into()),
            mime_type: Some("application/pdf".into()),
            file_size: Some(1200),
            file_hash: Some("sha256-pod-v1".into()),
        },
        Some(fixture.owner_user.id),
    )
    .await
    .unwrap()
    .expect("document should be created");

    db::dispatch::update_load_document(
        &pool,
        created.id,
        &db::dispatch::UpsertLoadDocumentParams {
            document_name: "Proof of delivery".into(),
            document_type: "delivery_pod".into(),
            file_path: "local://load-documents/p13/pod-v2.pdf".into(),
            storage_provider: "local".into(),
            original_name: Some("pod-v2.pdf".into()),
            mime_type: Some("application/pdf".into()),
            file_size: Some(1800),
            file_hash: Some("sha256-pod-v2".into()),
        },
        Some(fixture.owner_user.id),
    )
    .await
    .unwrap()
    .expect("document should be updated");

    let versions = sqlx::query_as::<_, (i32, String, String)>(
        "SELECT version_number, storage_key, file_name
         FROM document_versions
         WHERE document_scope = 'load_document' AND document_scope_id = $1
         ORDER BY version_number ASC",
    )
    .bind(created.id.to_string())
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(versions.len(), 2);
    assert_eq!(versions[0].0, 1);
    assert_eq!(versions[0].1, "local://load-documents/p13/pod-v1.pdf");
    assert_eq!(versions[0].2, "pod-v1.pdf");
    assert_eq!(versions[1].0, 2);
    assert_eq!(versions[1].1, "local://load-documents/p13/pod-v2.pdf");

    let audit_events = sqlx::query_scalar::<_, String>(
        "SELECT event_type
         FROM document_audit_events
         WHERE document_scope = 'load_document' AND document_scope_id = $1
         ORDER BY id ASC",
    )
    .bind(created.id.to_string())
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(audit_events, vec!["uploaded", "revision_uploaded"]);

    assert!(
        db::dispatch::load_has_payment_blocking_documents(&pool, fixture.load_id)
            .await
            .unwrap()
    );

    db::dispatch::review_load_document(
        &pool,
        created.id,
        db::dispatch::DocumentReviewDecision::Approve,
        Some("POD reviewed."),
        Some(fixture.owner_user.id),
    )
    .await
    .unwrap()
    .expect("document review should succeed");

    assert!(
        !db::dispatch::load_has_payment_blocking_documents(&pool, fixture.load_id)
            .await
            .unwrap()
    );
}
