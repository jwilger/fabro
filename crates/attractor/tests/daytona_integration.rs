//! Integration tests for `DaytonaExecutionEnvironment`.
//!
//! These tests require a `DAYTONA_API_KEY` environment variable and network access.
//! Run with: `cargo test --package attractor -- --ignored daytona`

use std::collections::HashMap;

use agent::ExecutionEnvironment;
use attractor::artifact::sync_artifacts_to_env;
use attractor::daytona_env::{DaytonaConfig, DaytonaExecutionEnvironment};

async fn create_env() -> DaytonaExecutionEnvironment {
    dotenvy::dotenv().ok();
    let client = daytona_sdk::Client::new()
        .await
        .expect("Failed to create Daytona client — is DAYTONA_API_KEY set?");
    DaytonaExecutionEnvironment::new(client, DaytonaConfig::default())
}

#[tokio::test]
#[ignore]
async fn daytona_exec_command() {
    let env = create_env().await;
    env.initialize().await.unwrap();

    let result = env
        .exec_command("echo hello", 30_000, None, None, None)
        .await
        .unwrap();
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("hello"));

    env.cleanup().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn daytona_exec_command_with_pipe() {
    let env = create_env().await;
    env.initialize().await.unwrap();

    let result = env
        .exec_command("echo hello world | wc -w", 30_000, None, None, None)
        .await
        .unwrap();
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.trim().contains('2'));

    env.cleanup().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn daytona_file_round_trip() {
    let env = create_env().await;
    env.initialize().await.unwrap();

    let test_path = "test_round_trip.txt";
    let content = "Hello from Daytona integration test!";

    // Write
    env.write_file(test_path, content).await.unwrap();

    // Exists
    assert!(env.file_exists(test_path).await.unwrap());

    // Read
    let read_back = env.read_file(test_path, None, None).await.unwrap();
    assert!(read_back.contains(content));

    // Delete
    env.delete_file(test_path).await.unwrap();
    assert!(!env.file_exists(test_path).await.unwrap());

    env.cleanup().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn daytona_full_lifecycle() {
    let env = create_env().await;

    // Initialize (creates sandbox + clones repo)
    env.initialize().await.unwrap();

    // Verify platform
    assert_eq!(env.platform(), "linux");

    // Verify working directory is accessible
    let result = env
        .exec_command("pwd", 10_000, None, None, None)
        .await
        .unwrap();
    assert_eq!(result.exit_code, 0);

    // List directory
    let entries = env.list_directory(".", None).await.unwrap();
    assert!(!entries.is_empty());

    // Cleanup (deletes sandbox)
    env.cleanup().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn daytona_snapshot_sandbox() {
    use attractor::daytona_env::{DaytonaSnapshotConfig, DaytonaSandboxConfig};

    dotenvy::dotenv().ok();
    let client = daytona_sdk::Client::new()
        .await
        .expect("Failed to create Daytona client — is DAYTONA_API_KEY set?");

    let config = DaytonaConfig {
        sandbox: DaytonaSandboxConfig {
            auto_stop_interval: Some(60),
            ..Default::default()
        },
        snapshot: Some(DaytonaSnapshotConfig {
            name: "attractor-test-snapshot".to_string(),
            cpu: Some(2),
            memory: Some(4),
            disk: Some(10),
            dockerfile: Some("FROM ubuntu:22.04\nRUN apt-get update && apt-get install -y ripgrep".to_string()),
        }),
    };

    let env = DaytonaExecutionEnvironment::new(client, config);
    env.initialize().await.unwrap();

    // Verify rg is available (installed by snapshot)
    let result = env
        .exec_command("rg --version", 10_000, None, None, None)
        .await
        .unwrap();
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("ripgrep"));

    env.cleanup().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn daytona_artifact_sync_uploads_and_rewrites_pointer() {
    let env = create_env().await;
    env.initialize().await.unwrap();

    // Create a local artifact file (simulating what offload_large_values produces)
    let dir = tempfile::tempdir().unwrap();
    let artifact_content = "x".repeat(150 * 1024); // 150KB
    let artifact_json = serde_json::json!(artifact_content);
    let artifact_file = dir.path().join("response.plan.json");
    std::fs::write(&artifact_file, serde_json::to_string(&artifact_json).unwrap()).unwrap();

    // Build updates with a file:// pointer (as offload_large_values would)
    let pointer = format!("file://{}", artifact_file.display());
    let mut updates = HashMap::new();
    updates.insert(
        "response.plan".to_string(),
        serde_json::json!(pointer),
    );

    // Sync — the local file doesn't exist in the Daytona sandbox, so it should upload
    sync_artifacts_to_env(&mut updates, &env).await.unwrap();

    // Pointer should be rewritten to the Daytona working directory
    let new_pointer = updates["response.plan"].as_str().unwrap();
    let expected_prefix = format!(
        "file://{}/.attractor/artifacts/",
        env.working_directory()
    );
    assert!(
        new_pointer.starts_with(&expected_prefix),
        "pointer should reference Daytona path, got: {new_pointer}"
    );

    // Verify the file actually exists in the sandbox by reading it back
    let remote_path = new_pointer.strip_prefix("file://").unwrap();
    assert!(
        env.file_exists(remote_path).await.unwrap(),
        "artifact file should exist in Daytona sandbox at {remote_path}"
    );

    let remote_content = env.read_file(remote_path, None, None).await.unwrap();
    assert!(
        remote_content.len() > 100 * 1024,
        "remote artifact should be >100KB, got {} bytes",
        remote_content.len()
    );

    env.cleanup().await.unwrap();
}
