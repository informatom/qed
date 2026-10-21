//! Publishes to the throwaway server from `dev/test-server/compose.yml`.
//!
//! These tests need that server, so they're skipped by default:
//!
//! ```sh
//! docker compose -f dev/test-server/compose.yml up -d
//! cargo test -p qed-core --test test_server -- --ignored --test-threads=1
//! ```

use std::io::{Read, Write};
use std::path::Path;

use qed_core::sftp::{self, Credentials, SftpError};
use qed_core::target::{SftpAuth, SftpTarget};
use qed_core::{generator, site};

fn target(host_key: Option<String>) -> SftpTarget {
    SftpTarget {
        name: "Test server".into(),
        host: "127.0.0.1".into(),
        port: 2222,
        user: "qed".into(),
        path: "/www".into(),
        auth: SftpAuth::Password,
        host_key,
        delete_stale: true,
    }
}

fn password() -> Credentials {
    Credentials::Password("qed".into())
}

/// The fingerprint the server presents, learned the way the app does: by
/// connecting without one.
async fn trusted_target() -> SftpTarget {
    match sftp::connect(&target(None), &password()).await {
        Err(SftpError::UnknownHostKey { fingerprint }) => target(Some(fingerprint)),
        Err(other) => panic!("is the test server running? {other}"),
        Ok(_) => panic!("connected without a trusted host key"),
    }
}

/// A plain HTTP GET against the test server's web container.
fn fetch(path: &str) -> String {
    let mut stream = std::net::TcpStream::connect("127.0.0.1:8080").unwrap();
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

#[tokio::test]
#[ignore = "needs dev/test-server"]
async fn asks_before_trusting_and_refuses_a_changed_key() {
    let target = trusted_target().await;
    assert!(target.host_key.as_deref().unwrap().starts_with("SHA256:"));

    let mut wrong = target.clone();
    wrong.host_key = Some("SHA256:not-the-real-key".into());
    let error = sftp::connect(&wrong, &password())
        .await
        .err()
        .expect("must refuse");
    assert!(matches!(error, SftpError::HostKeyChanged { .. }), "{error}");
}

#[tokio::test]
#[ignore = "needs dev/test-server"]
async fn rejects_a_wrong_password() {
    let target = trusted_target().await;
    let error = sftp::connect(&target, &Credentials::Password("wrong".into()))
        .await
        .err()
        .expect("must refuse");
    assert!(matches!(error, SftpError::AuthFailed { .. }), "{error}");
}

#[tokio::test]
#[ignore = "needs dev/test-server"]
async fn publishes_only_what_changed() {
    let target = trusted_target().await;
    let message = sftp::test(&target, &password()).await.unwrap();
    assert!(message.starts_with("Connected."), "{message}");

    // Build the demo site into a temporary folder.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/demo-site");
    let mut demo = site::open(&root).unwrap();
    let out = tempfile::tempdir().unwrap();
    generator::build_into(&demo, out.path()).unwrap();

    let first = sftp::publish(&target, &password(), out.path(), |_| {})
        .await
        .unwrap();
    assert!(first.uploaded + first.unchanged >= 5, "{first:?}");

    // The site is online.
    let home = fetch("/");
    assert!(home.starts_with("HTTP/1.1 200"), "{home}");
    assert!(home.contains("Two Mics and a Plan"), "{home}");

    // Nothing changed: nothing is uploaded.
    let again = sftp::publish(&target, &password(), out.path(), |_| {})
        .await
        .unwrap();
    assert_eq!(again.uploaded, 0, "{again:?}");

    // One page changed and one is gone: one upload, one delete.
    demo.pages.retain(|page| page.path != "imprint");
    demo.pages[0].content.front.title = "Welcome".into();
    let out2 = tempfile::tempdir().unwrap();
    generator::build_into(&demo, out2.path()).unwrap();
    let mut stages = Vec::new();
    let third = sftp::publish(&target, &password(), out2.path(), |p| stages.push(p.stage))
        .await
        .unwrap();
    assert!(third.uploaded >= 1, "{third:?}");
    assert_eq!(third.deleted, 1, "{third:?}");
    assert!(stages.contains(&sftp::Stage::Deleting));
    assert!(fetch("/imprint/").starts_with("HTTP/1.1 404"));

    // Put the full demo site back, for the next run and for looking at it.
    sftp::publish(&target, &password(), out.path(), |_| {})
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "needs dev/test-server"]
async fn creates_only_missing_folders() {
    let mut target = trusted_target().await;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/demo-site");
    let out = tempfile::tempdir().unwrap();
    generator::build_into(&site::open(&root).unwrap(), out.path()).unwrap();

    // Two levels that don't exist yet below /www.
    target.path = "/www/nested/site/".into();
    let message = sftp::test(&target, &password()).await.unwrap();
    assert!(message.contains("isn't there"), "{message}");
    assert!(
        message.contains("nearest folder that exists is /www."),
        "{message}"
    );
    sftp::publish(&target, &password(), out.path(), |_| {})
        .await
        .unwrap();
    assert!(fetch("/nested/site/").contains("Two Mics and a Plan"));

    // Now it exists; publishing again must not try to create anything.
    let message = sftp::test(&target, &password()).await.unwrap();
    assert!(message.contains("earlier publish"), "{message}");
    sftp::publish(&target, &password(), out.path(), |_| {})
        .await
        .unwrap();
}
