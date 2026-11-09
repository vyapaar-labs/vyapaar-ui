//! Stage 4 E2E tests for the CLI jobs runtime
//! (`vyapaar jobs add ... -> tick -> journal verified`, PLAN-0001 Phase 4).
//!
//! All tests run in isolated `TempDir` homes; nothing touches `~/.vyapaar`.

use std::path::Path;

use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use tempfile::TempDir;
// Trait must be in scope: `StrategyBridgeRunner::run_backtest` comes from
// `vyapaar_agent_chat_tools::ports::strategy::StrategyRunner`.
use vyapaar_agent_chat_tools::ports::strategy::StrategyRunner;
use vyapaar_cli::jobs_runtime::{
    AddRequest, JobsRuntime, RunDisposition, boot_recover, mint_run_id, short_id,
};
use vyapaar_cli::strategy_bridge::StrategyBridgeRunner;
use vyapaar_runtime_api_rest::FileJournalWriter;
use vyapaar_runtime_jobs::{
    BoundedWorkerPool, FileJobRegistry, JobStatus, MissedPolicy, WorkerPoolConfig,
};

fn temp_home() -> TempDir {
    TempDir::new().unwrap()
}

fn add_request(goal: &str, schedule: Option<&str>) -> AddRequest {
    AddRequest {
        goal: goal.into(),
        schedule: schedule.map(str::to_string),
        max_turns: 20,
        max_inr: None,
        max_usd: None,
        max_tokens: Some(4_000),
        max_tool_calls: None,
        max_data_calls: None,
        missed_policy: MissedPolicy::Skip,
        approval_manual: false,
        strategy: Some("sma_crossover".into()),
        symbol: Some("NSE:RELIANCE-EQ".into()),
        capital: Some(Decimal::new(1_000_000, 0)),
        params: None,
        success: None,
        read_only: true,
        once: false,
    }
}

/// Rewinds a job's `created_at` so its `* * * * *` schedule is past-due
/// deterministically (a freshly created cron job fires on the next minute
/// boundary, which would make the tick racy).
fn seed_registered_ago(home: &Path, spec_id: uuid::Uuid, ago: Duration) {
    let mut registry = FileJobRegistry::load_or_create(home.join("jobs.json")).unwrap();
    let job = registry.get_job_mut(&spec_id).unwrap();
    job.created_at = Utc::now() - ago;
    job.last_run_at = None;
    registry.save().unwrap();
}

fn read_manifest(dir: &std::path::Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(dir.join("manifest.json")).unwrap()).unwrap()
}

fn event_types(dir: &std::path::Path) -> Vec<String> {
    std::fs::read_to_string(dir.join("events.ndjson"))
        .unwrap()
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let event: serde_json::Value = serde_json::from_str(line).unwrap();
            event["type"].as_str().unwrap().to_string()
        })
        .collect()
}

#[test]
fn add_run_tick_and_journal_roundtrip() {
    let temp = temp_home();
    let home = temp.path().to_path_buf();
    let runtime = JobsRuntime::open(&home).unwrap();
    let spec = runtime
        .add(&add_request(
            "Scan momentum breaks in nifty midcaps",
            Some("* * * * *"),
        ))
        .unwrap();
    seed_registered_ago(&home, spec.id, Duration::hours(2));

    // CLI-level roundtrip: add -> tick executes the due job -> journal verified.
    let report = runtime.tick().unwrap();
    assert_eq!(report.executed.len(), 1, "the created job must be due");
    assert!(report.held.is_empty());
    assert!(report.pending_until.is_empty());
    let (job_short, receipt) = &report.executed[0];
    assert_eq!(job_short, &short_id(&spec));
    assert_eq!(receipt.disposition, RunDisposition::Completed);
    assert!(
        receipt.error.is_none(),
        "unexpected run error: {:?}",
        receipt.error
    );
    let journal = receipt.journal_dir.as_ref().expect("journal dir for a run");

    // manifest.json — job-shaped, terminal state, stamped exclusively in the
    // manifest (never inside event payloads).
    let manifest = read_manifest(journal);
    assert_eq!(manifest["kind"], "job");
    assert_eq!(manifest["status"], "completed");
    assert_eq!(manifest["job_id"], spec.id.simple().to_string().as_str());
    assert_eq!(manifest["goal"], "Scan momentum breaks in nifty midcaps");
    assert_eq!(manifest["strategy"], "sma_crossover");
    assert_eq!(manifest["symbol"], "NSE:RELIANCE-EQ");
    assert_eq!(manifest["capital"], "1000000");
    assert!(manifest["created_at"].as_str().is_some());
    assert!(manifest["started_at"].as_str().is_some());
    assert!(manifest["finished_at"].as_str().is_some());
    assert!(manifest.get("error").is_none());

    // events.ndjson — deterministic seq/type event stream, no wall clock.
    assert_eq!(
        event_types(journal),
        vec![
            "run.queued".to_string(),
            "run.started".to_string(),
            "run.result".to_string(),
            "run.finalized".to_string(),
        ]
    );
    let events: Vec<serde_json::Value> = std::fs::read_to_string(journal.join("events.ndjson"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(events[1]["worker"], "cli");
    assert!(events[2]["report"].is_object(), "engine report journaled");

    // trades.json / metrics.json — engine aggregates only (see gap 0002).
    let trades: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(journal.join("trades.json")).unwrap())
            .unwrap();
    assert!(
        trades
            .get("total_trades")
            .and_then(|v| v.as_u64())
            .is_some()
    );
    let metrics: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(journal.join("metrics.json")).unwrap())
            .unwrap();
    assert!(metrics.get("sharpe_ratio").is_some());
    assert!(metrics.get("net_pnl").is_some());

    // `jobs logs` sees the journaled run.
    let runs = runtime.journal_runs(&spec).unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].manifest["status"], "completed");
}

#[test]
fn once_job_completes_and_next_tick_skips() {
    let temp = temp_home();
    let home = temp.path().to_path_buf();
    let runtime = JobsRuntime::open(&home).unwrap();
    let mut req = add_request("Sweep the universe once", Some("* * * * *"));
    req.once = true;
    let spec = runtime.add(&req).unwrap();
    seed_registered_ago(&home, spec.id, Duration::hours(2));

    let report = runtime.tick().unwrap();
    assert_eq!(report.executed.len(), 1);
    assert_eq!(report.executed[0].1.disposition, RunDisposition::Completed);

    let registry = FileJobRegistry::load_or_create(home.join("jobs.json")).unwrap();
    let job = registry.get_job(&spec.id).unwrap();
    assert_eq!(job.status, JobStatus::Completed);
    assert!(job.active_lease.is_none());

    // Completed once-jobs are skipped by the next tick.
    let second = runtime.tick().unwrap();
    assert!(second.executed.is_empty());
}

#[test]
fn failed_strategy_run_journals_failure() {
    let temp = temp_home();
    let runtime = JobsRuntime::open(temp.path()).unwrap();
    let mut req = add_request("Bad strategy run", None);
    req.strategy = Some("no_such_strategy".into());
    let spec = runtime.add(&req).unwrap();

    let receipt = runtime.run(&spec, "manual").unwrap();
    assert_eq!(receipt.disposition, RunDisposition::Failed);
    let journal = receipt.journal_dir.as_ref().unwrap();
    let manifest = read_manifest(journal);
    assert_eq!(manifest["status"], "failed");
    assert_eq!(manifest["error"]["code"], "internal_error");
    assert_eq!(
        event_types(journal),
        vec![
            "run.queued".to_string(),
            "run.started".to_string(),
            "run.failed".to_string(),
            "run.finalized".to_string(),
        ]
    );
}

#[test]
fn run_refusals_for_missing_payload_paused_and_completed() {
    let temp = temp_home();
    let runtime = JobsRuntime::open(temp.path()).unwrap();

    // Goal-only job: no executable payload.
    let mut req = add_request("Research only", None);
    req.strategy = None;
    req.symbol = None;
    let spec = runtime.add(&req).unwrap();
    let err = runtime
        .run(&runtime.resolve_job(&short_id(&spec)).unwrap(), "manual")
        .unwrap_err();
    assert!(err.to_string().contains("no executable payload"));

    // Paused job.
    let spec = runtime
        .add(&add_request("Held while paused", None))
        .unwrap();
    runtime.pause(&short_id(&spec)).unwrap();
    let err = runtime
        .run(&runtime.resolve_job(&short_id(&spec)).unwrap(), "manual")
        .unwrap_err();
    assert!(err.to_string().contains("is paused"));
    runtime.resume(&short_id(&spec)).unwrap();

    // Run-once job completes and is refused afterwards.
    let mut req = add_request("One shot only", None);
    req.once = true;
    let spec = runtime.add(&req).unwrap();
    let receipt = runtime
        .run(&runtime.resolve_job(&short_id(&spec)).unwrap(), "manual")
        .unwrap();
    assert_eq!(receipt.disposition, RunDisposition::Completed);
    let err = runtime
        .run(&runtime.resolve_job(&short_id(&spec)).unwrap(), "manual")
        .unwrap_err();
    assert!(err.to_string().contains("already completed"));
}

#[test]
fn budget_breach_routes_to_approvals_and_refuses() {
    let temp = temp_home();
    let runtime = JobsRuntime::open(temp.path()).unwrap();
    let mut req = add_request("Tiny tool budget", None);
    req.max_tool_calls = Some(0);
    let spec = runtime.add(&req).unwrap();

    // The first run executes, but recording its tool call breaches the budget
    // and moves the job to the approvals queue.
    let receipt = runtime
        .run(&runtime.resolve_job(&short_id(&spec)).unwrap(), "manual")
        .unwrap();
    assert_eq!(receipt.job_status, JobStatus::PendingApproval);
    let queue = runtime.approvals().unwrap();
    assert_eq!(queue.len(), 1);
    assert_eq!(queue[0].id, spec.id);

    // The breached job refuses further runs until budgets are refilled.
    let refreshed = runtime.resolve_job(&short_id(&spec)).unwrap();
    let err = runtime.run(&refreshed, "manual").unwrap_err();
    assert!(err.to_string().contains("budget exceeded"));
    assert!(err.to_string().contains("approvals queue"));

    // A deny parks the job (registry has no terminal-reject state) and
    // clears the queue.
    runtime.deny(&short_id(&spec)).unwrap();
    assert!(runtime.approvals().unwrap().is_empty());
    let denied = runtime.resolve_job(&short_id(&spec)).unwrap();
    assert_eq!(denied.status, JobStatus::Paused);
}

#[test]
fn manual_approval_policy_gates_every_run() {
    let temp = temp_home();
    let home = temp.path().to_path_buf();
    let runtime = JobsRuntime::open(&home).unwrap();
    let mut req = add_request("Human in the loop", Some("* * * * *"));
    req.approval_manual = true;
    let spec = runtime.add(&req).unwrap();
    seed_registered_ago(&home, spec.id, Duration::hours(2));

    // Tick holds the manual-policy job behind the approval gate.
    let report = runtime.tick().unwrap();
    assert!(report.executed.is_empty());
    assert_eq!(report.held.len(), 1);
    assert!(report.held[0].1.contains("manual approval policy"));
    let err = runtime
        .run(&runtime.resolve_job(&short_id(&spec)).unwrap(), "manual")
        .unwrap_err();
    assert!(err.to_string().contains("awaiting human approval"));

    // Approve -> Active -> the operator's manual run is admitted.
    runtime.approve(&short_id(&spec)).unwrap();
    let spec_after_approve = runtime.resolve_job(&short_id(&spec)).unwrap();
    let receipt = runtime.run(&spec_after_approve, "manual").unwrap();
    assert_eq!(receipt.disposition, RunDisposition::Completed);

    // After every run the manual gate holds the job again.
    assert_eq!(receipt.job_status, JobStatus::PendingApproval);
    assert_eq!(runtime.approvals().unwrap().len(), 1);
    // And the next tick never auto-fires it.
    let second = runtime.tick().unwrap();
    assert!(
        second.executed.is_empty(),
        "manual jobs are never auto-fired"
    );
}

#[test]
fn saturated_pool_refuses_run_without_journal() {
    let temp = temp_home();
    let runtime = JobsRuntime::open_with_pool(
        temp.path(),
        BoundedWorkerPool::new(WorkerPoolConfig::new(1, 0, 1)),
    )
    .unwrap();
    let spec = runtime
        .add(&add_request("Blocked by backpressure", None))
        .unwrap();

    // Occupy the pool's single permit deterministically on a second thread.
    let pool = runtime.pool();
    let (park_tx, park_rx) = std::sync::mpsc::channel::<()>();
    let (parked_tx, parked_rx) = std::sync::mpsc::channel::<()>();
    let holder = std::thread::spawn(move || {
        let blocked = parked_tx;
        let io_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        io_runtime.block_on(async move {
            let _ = pool
                .dispatch(move || {
                    let _ = blocked.send(());
                    let _ = park_rx.recv();
                })
                .await;
        });
    });
    parked_rx.recv().unwrap();

    let receipt = runtime
        .run(&runtime.resolve_job(&short_id(&spec)).unwrap(), "manual")
        .unwrap();
    assert_eq!(receipt.disposition, RunDisposition::Backpressured);
    assert!(
        receipt
            .error
            .as_deref()
            .unwrap_or("")
            .contains("backpressure"),
        "refusal must name backpressure: {:?}",
        receipt.error
    );
    assert!(receipt.run_id.is_none(), "no journal exists for a refusal");

    // The lease was released and the job rests Active for a retry.
    let after = runtime.resolve_job(&short_id(&spec)).unwrap();
    assert_eq!(after.status, JobStatus::Active);
    assert!(after.active_lease.is_none());

    park_tx.send(()).unwrap();
    holder.join().unwrap();
}

#[test]
fn agent_tool_calls_journal_tool_runs() {
    let temp = temp_home();
    let home = temp.path().to_path_buf();
    let runner = StrategyBridgeRunner::with_journal_home(home.clone());
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let report = rt
        .block_on(runner.run_backtest(
            "sma_crossover",
            "NSE:RELIANCE-EQ",
            Decimal::new(1_000_000, 0),
            None,
        ))
        .unwrap();
    assert!(report.get("total_trades").is_some());
    let failure = rt
        .block_on(runner.run_backtest(
            "no_such_strategy",
            "NSE:RELIANCE-EQ",
            Decimal::new(1_000_000, 0),
            None,
        ))
        .unwrap_err();

    let journals = home.join("journals");
    let mut tool_runs = Vec::new();
    for entry in std::fs::read_dir(&journals).unwrap() {
        let dir = entry.unwrap().path();
        let manifest: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(dir.join("manifest.json")).unwrap())
                .unwrap();
        tool_runs.push((dir, manifest));
    }
    assert_eq!(tool_runs.len(), 2, "one journal per agent tool call");
    for (dir, manifest) in &tool_runs {
        assert_eq!(manifest["kind"], "tool");
        assert_eq!(manifest["tool"], "run_backtest");
        assert_eq!(manifest["input"]["capital"], "1000000");
        assert!(
            event_types(dir).starts_with(&["tool.started".to_string(),]),
            "tool runs journal their deterministic event trail"
        );
    }

    let successful = tool_runs
        .iter()
        .find(|(_, m)| m["status"] == "completed")
        .expect("successful tool call journaled");
    assert_eq!(successful.1["input"]["strategy"], "sma_crossover");
    assert_eq!(successful.1["error"]["code"], serde_json::Value::Null);
    assert!(
        std::fs::read_to_string(successful.0.join("metrics.json"))
            .unwrap()
            .contains("total_trades"),
        "the engine result is the tool run's metrics.json"
    );
    let failed = tool_runs
        .iter()
        .find(|(_, m)| m["status"] == "failed")
        .expect("failed tool call journaled");
    assert_eq!(failed.1["input"]["strategy"], "no_such_strategy");
    assert_eq!(failed.1["error"]["code"], "internal_error");
    // Tool error detail names the engine failure.
    assert!(
        failed.1["error"]["message"].is_string(),
        "failure journaled: {failure:?}"
    );
}

#[test]
fn boot_recover_normalizes_interrupted_job_journal() {
    let temp = temp_home();
    let home = temp.path().to_path_buf();
    std::fs::create_dir_all(home.join("journals")).unwrap();

    // Simulate a process crash mid-run: a running job manifest without a
    // terminal snapshot.
    let run_id = mint_run_id().unwrap();
    let writer = FileJournalWriter::create_from_value(
        &home.join("journals"),
        &run_id,
        &serde_json::json!({
            "manifest_version": 1,
            "kind": "job",
            "run_id": run_id.as_str(),
            "job_id": "deadbeefdeadbeefdeadbeefdeadbeef",
            "status": "running",
            "created_at": "2026-11-09T09:00:00Z",
            "started_at": "2026-11-09T09:00:01Z"
        }),
    )
    .unwrap();
    drop(writer);

    assert_eq!(boot_recover(Some(&home)), 1);
    let manifest = read_manifest(&home.join("journals").join(run_id.as_str()));
    assert_eq!(manifest["status"], "failed");
    assert_eq!(manifest["error"]["code"], "internal_error");
    assert!(manifest["finished_at"].as_str().is_some());
    // Idempotent: the second sweep finds nothing left.
    assert_eq!(boot_recover(Some(&home)), 0);
    // A never-created home is a no-op.
    let absent = home.join("does-not-exist");
    assert_eq!(boot_recover(Some(&absent)), 0);
}

#[test]
fn legacy_jobs_json_is_archived_and_registry_recovers() {
    let temp = temp_home();
    let home = temp.path().to_path_buf();
    // A pre-runtime flat stub file in an unparseable format.
    std::fs::write(
        home.join("jobs.json"),
        r#"{"randomized":"stubbing-manifest-not-a-list"}"#,
    )
    .unwrap();

    let runtime = JobsRuntime::open(&home).unwrap();
    runtime
        .add(&add_request("Created after migration", None))
        .unwrap();

    assert!(home.join("jobs.json.legacy").exists());
    let registry = FileJobRegistry::load_or_create(home.join("jobs.json")).unwrap();
    let goals: Vec<String> = registry.list_jobs().into_iter().map(|j| j.goal).collect();
    assert_eq!(goals, vec!["Created after migration".to_string()]);
}

#[test]
fn resolve_job_accepts_prefixes_and_rejects_ambiguity() {
    let temp = temp_home();
    let runtime = JobsRuntime::open(temp.path()).unwrap();
    let only = runtime.add(&add_request("Unique job", None)).unwrap();
    let resolved = runtime.resolve_job(&short_id(&only)).unwrap();
    assert_eq!(resolved.id, only.id);
    let err = runtime.resolve_job("zzzzzzzz").unwrap_err();
    assert!(err.to_string().contains("not found"));
    let err = runtime.resolve_job("").unwrap_err();
    assert!(err.to_string().contains("empty"));
}
