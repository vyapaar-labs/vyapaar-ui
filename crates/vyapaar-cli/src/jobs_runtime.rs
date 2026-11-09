//! Runtime job plumbing for `vyapaar jobs` (PLAN-0001 Stage 4).
//!
//! Wires the CLI command tree to the runtime stack built in Stages 1–3:
//!
//! - [`vyapaar_runtime_jobs::FileJobRegistry`] is the persistent job store
//!   (`~/.vyapaar/jobs.json`, atomic tmp-rename writes, fencing leases,
//!   approvals queue).
//! - [`vyapaar_runtime_jobs::BoundedWorkerPool`] is the bounded execution
//!   pool (HTTP 429 backpressure, panic isolation).
//! - [`vyapaar_runtime_api::RunIdGenerator`] mints the 26-char Crockford
//!   Base32 monotonic run ids and [`vyapaar_runtime_api_rest::FileJournalWriter`]
//!   maintains the per-run journal directory
//!   (`~/.vyapaar/journals/<run_id>/{manifest.json,events.ndjson,trades.json,metrics.json}`).
//!
//! The manifest of a job run is a job-shaped journal manifest (`kind: "job"`)
//! that shares the `run_id`/`status` conventions of the REST `RunManifest`
//! without carrying strategy IR. Crash recovery sweeps both shapes
//! (`recover_interrupted_runs`).
//!
//! Event payloads are deterministic (sequence number + event type + engine
//! output); wall-clock timestamps live only in the manifest snapshots, so a
//! job run replays byte-for-byte from its journal.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use vyapaar_runtime_api::{ErrorCode, ErrorDetail, RunEvent, RunId, RunIdGenerator, RunStatus};
use vyapaar_runtime_api_rest::{FileJournalWriter, recover_interrupted_runs};
use vyapaar_runtime_jobs::{
    ApprovalPolicy, BoundedWorkerPool, BudgetUsage, CronSchedule, FileJobRegistry, JobBudgets,
    JobSpec, JobStatus, MissedPolicy, PoolError, RegistryError, WorkerPoolConfig,
};
use vyapaar_ui_core::strategy::StrategyBridge;

use crate::runner::CliError;

/// Worker identity stamped into fencing leases and journal events.
const WORKER_ID: &str = "cli";
/// Lease TTL: a CLI run must finish within this window or the lease lapses.
const LEASE_TTL_SECONDS: u64 = 900;
/// Metadata key recording the trigger class of a job ("manual" | "cron").
const TRIGGER_KEY: &str = "trigger";
/// Trigger class of schedule-less jobs; `tick` never auto-fires them.
const MANUAL_TRIGGER: &str = "manual";
/// Trigger class of scheduled jobs fired by `tick`.
const CRON_TRIGGER: &str = "cron";
/// Placeholder cron satisfying the typed `JobSpec.schedule` field of manual jobs.
const MANUAL_PLACEHOLDER_CRON: &str = "* * * * *";

/// Whether a dispatched run actually produced a journal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunDisposition {
    /// The worker completed the run and finalized its journal.
    Completed,
    /// The worker ran and marked the journal failed (`RunStatus::Failed`).
    Failed,
    /// The run could not be admitted to the worker pool (HTTP 429
    /// backpressure): no journal exists for it.
    Backpressured,
}

/// Result of one job execution attempt.
#[derive(Debug, Clone)]
pub struct RunReceipt {
    /// Which journal terminal state was reached.
    pub disposition: RunDisposition,
    /// The run id (`None` only for [`RunDisposition::Backpressured`]).
    pub run_id: Option<RunId>,
    /// Error detail for failed / refused runs.
    pub error: Option<String>,
    /// Resulting registry status after usage was recorded.
    pub job_status: JobStatus,
    /// Journal directory, when a run was journaled.
    pub journal_dir: Option<PathBuf>,
    /// Wall-clock duration of the execution.
    pub duration_ms: u128,
}

/// The executable payload of a job: a strategy backtest dispatched onto the
/// bounded worker pool. A goal-only job (no `--strategy`) has no payload and
/// is refused instead of silently executing nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyPayload {
    pub strategy: String,
    pub symbol: String,
    pub capital: Decimal,
    pub params: Option<BTreeMap<String, Decimal>>,
}

impl StrategyPayload {
    /// Reads `strategy`, `symbol`, `capital` and `params` from the job spec's
    /// metadata (written by `vyapaar jobs add --strategy ...`).
    pub fn from_metadata(spec: &JobSpec) -> Option<Self> {
        let strategy = spec.metadata.get("strategy")?.trim().to_string();
        if strategy.is_empty() {
            return None;
        }
        let symbol = spec
            .metadata
            .get("symbol")
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let capital = spec
            .metadata
            .get("capital")
            .and_then(|c| c.trim().parse::<Decimal>().ok())
            .unwrap_or(Decimal::new(1_000_000, 0));
        let params = spec
            .metadata
            .get("params")
            .and_then(|p| serde_json::from_str::<BTreeMap<String, Decimal>>(p).ok());
        Some(Self {
            strategy,
            symbol,
            capital,
            params,
        })
    }
}

/// CLI-side arguments for `vyapaar jobs add`, independent of clap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddRequest {
    pub goal: String,
    pub schedule: Option<String>,
    pub max_turns: usize,
    pub max_inr: Option<Decimal>,
    pub max_usd: Option<Decimal>,
    pub max_tokens: Option<usize>,
    pub max_tool_calls: Option<usize>,
    pub max_data_calls: Option<usize>,
    pub missed_policy: MissedPolicy,
    /// Approval policy: `auto` runs within budget; `manual` holds every run.
    pub approval_manual: bool,
    pub strategy: Option<String>,
    pub symbol: Option<String>,
    pub capital: Option<Decimal>,
    /// Strategy parameter overrides as JSON.
    pub params: Option<String>,
    /// Optional free-form success criteria recorded in the job metadata.
    pub success: Option<String>,
    pub read_only: bool,
    pub once: bool,
}

/// Aggregated result of a `vyapaar jobs tick`.
#[derive(Debug, Clone, Default)]
pub struct TickReport {
    /// Interrupted run journals normalized by the crash-recovery sweeper.
    pub recovered_runs: usize,
    /// Fencing leases that lapsed and were swept on this tick.
    pub swept_leases: usize,
    /// Jobs held back requiring human approval (`PendingApproval`).
    pub held: Vec<(String, String)>,
    /// Jobs whose next firing is still in the future.
    pub pending_until: Vec<(String, String)>,
    /// Jobs dispatched due to a schedule fire, with their run receipt.
    pub executed: Vec<(String, RunReceipt)>,
}

/// Runtime facade: registry + worker pool + journaling, rooted at `home`.
pub struct JobsRuntime {
    home: PathBuf,
    pool: std::sync::Arc<BoundedWorkerPool>,
}

impl JobsRuntime {
    /// Opens (and creates) the runtime areas under `home`.
    pub fn open(home: &Path) -> Result<Self, CliError> {
        std::fs::create_dir_all(home)
            .and_then(|_| std::fs::create_dir_all(home.join("journals")))
            .map_err(|e| CliError::Execution(format!("cannot create Vyapaar home: {e}")))?;
        Ok(Self {
            home: home.to_path_buf(),
            pool: std::sync::Arc::new(BoundedWorkerPool::new(WorkerPoolConfig::default())),
        })
    }

    /// Opens the runtime with an explicit worker pool (perf/limits tests use
    /// hand-tuned pool configs to reproduce backpressure deterministically).
    pub fn open_with_pool(home: &Path, pool: BoundedWorkerPool) -> Result<Self, CliError> {
        std::fs::create_dir_all(home)
            .and_then(|_| std::fs::create_dir_all(home.join("journals")))
            .map_err(|e| CliError::Execution(format!("cannot create Vyapaar home: {e}")))?;
        Ok(Self {
            home: home.to_path_buf(),
            pool: std::sync::Arc::new(pool),
        })
    }

    /// The runtime home (`~/.vyapaar` by default).
    pub fn home(&self) -> &Path {
        &self.home
    }

    /// Where run journals live (`~/.vyapaar/journals`).
    pub fn journals_dir(&self) -> PathBuf {
        self.home.join("journals")
    }

    /// The shared bounded worker pool.
    pub fn pool(&self) -> std::sync::Arc<BoundedWorkerPool> {
        self.pool.clone()
    }

    /// Loads (or creates) the persistent job registry with a legacy-format
    /// migration: an unreadable `jobs.json` (pre-runtime flat stub entries) is
    /// archived as `jobs.json.legacy` and execution starts on a fresh store.
    pub fn registry(&self) -> Result<FileJobRegistry, CliError> {
        let path = self.home.join("jobs.json");
        match FileJobRegistry::load_or_create(&path) {
            Ok(registry) => Ok(registry),
            Err(RegistryError::Serialization(_)) => {
                let legacy = path.with_extension("json.legacy");
                let _ = std::fs::rename(&path, &legacy);
                FileJobRegistry::load_or_create(&path).map_err(into_cli)
            }
            Err(other) => Err(into_cli(other)),
        }
    }

    /// Resolves a job id token: the full UUID, its `simple()` form, or a
    /// unique prefix (the 8-hex shorthand shown by `jobs list`). Ambiguous or
    /// missing matches are rejected.
    pub fn resolve_job(&self, token: &str) -> Result<JobSpec, CliError> {
        let token = token.trim().to_lowercase();
        if token.is_empty() {
            return Err(CliError::Execution("job id cannot be empty".to_string()));
        }
        let registry = self.registry()?;
        let mut matches: Vec<JobSpec> = registry
            .list_jobs()
            .into_iter()
            .filter(|spec| {
                let simple = spec.id.simple().to_string();
                simple.starts_with(&token) || spec.id.to_string().starts_with(&token)
            })
            .collect();
        match matches.pop() {
            Some(spec) if matches.is_empty() => Ok(spec),
            Some(_) => Err(CliError::Execution(format!(
                "job id prefix '{token}' is ambiguous; pass a longer prefix"
            ))),
            None => Err(CliError::Execution(format!("job '{token}' not found"))),
        }
    }

    /// Registers a job specification (`vyapaar jobs add`).
    pub fn add(&self, req: &AddRequest) -> Result<JobSpec, CliError> {
        if req.goal.trim().is_empty() {
            return Err(CliError::Execution("job goal cannot be empty".to_string()));
        }
        // Schedule-less jobs are manual triggers: tick never auto-fires them.
        // The placeholder cron below satisfies the typed `JobSpec.schedule`
        // field; the `trigger` metadata marks the manual class.
        let (schedule, trigger) = match &req.schedule {
            Some(expr) => (
                CronSchedule::new(expr.as_str())
                    .map_err(|e| CliError::Execution(format!("invalid cron schedule: {e}")))?,
                CRON_TRIGGER,
            ),
            None => (
                CronSchedule::new(MANUAL_PLACEHOLDER_CRON).expect("placeholder cron is valid"),
                MANUAL_TRIGGER,
            ),
        };
        let mut spec = JobSpec::new(req.goal.trim(), schedule)
            .with_goal(req.goal.trim())
            .with_missed_policy(req.missed_policy)
            .with_read_only(req.read_only)
            .with_once(req.once);
        spec.approval_policy = if req.approval_manual {
            ApprovalPolicy::Manual
        } else {
            ApprovalPolicy::Auto
        };
        spec.budgets = budgets_from_request(req);
        spec.metadata
            .insert(TRIGGER_KEY.into(), trigger.to_string());
        for (key, value) in [
            ("strategy", req.strategy.as_deref()),
            ("symbol", req.symbol.as_deref()),
            ("success", req.success.as_deref()),
        ] {
            if let Some(value) = value {
                spec.metadata.insert(key.into(), value.into());
            }
        }
        if let Some(capital) = req.capital {
            spec.metadata.insert("capital".into(), capital.to_string());
        }
        if let Some(params) = req.params.as_deref() {
            if serde_json::from_str::<BTreeMap<String, Decimal>>(params).is_err() {
                return Err(CliError::Execution(
                    "invalid --params JSON: expected an object of decimal values".to_string(),
                ));
            }
            spec.metadata.insert("params".into(), params.into());
        }
        spec.next_run_at = match trigger {
            CRON_TRIGGER => spec.schedule.next_execution(Utc::now()),
            _ => None,
        };

        let mut registry = self.registry()?;
        registry.add_job(spec.clone()).map_err(into_cli)?;
        Ok(spec)
    }

    /// Executes one job run now (`vyapaar jobs run <id>`).
    ///
    /// Sequence (ADR 0017 / Jesse's fencing pattern): claim a fencing lease ->
    /// admit the run payload to the bounded pool -> journal
    /// queued/started/result events -> release the lease and record budget
    /// usage (breaches move the job to the approvals queue) -> advance the
    /// cron cursor.
    pub fn run(&self, spec: &JobSpec, trigger: &str) -> Result<RunReceipt, CliError> {
        match spec.status {
            JobStatus::PendingApproval => {
                // A PendingApproval job is held either for policy (awaiting a
                // human) or because a budget breach parked it; report the root
                // cause instead of the wrapper state.
                return Err(match spec.budgets.validate(&spec.usage) {
                    Err(breach) => CliError::Execution(format!(
                        "budget exceeded for job '{}': {breach}; it is held in \
                         the approvals queue until limits are raised \
                         (`vyapaar jobs approvals`)",
                        spec.name
                    )),
                    Ok(()) => CliError::Execution(format!(
                        "job '{}' is awaiting human approval; review with \
                         `vyapaar jobs approvals`",
                        spec.name
                    )),
                });
            }
            JobStatus::Paused => {
                return Err(CliError::Execution(format!(
                    "job '{}' is paused; resume it with `vyapaar jobs resume {}`",
                    spec.name,
                    short_id(spec)
                )));
            }
            JobStatus::Completed => {
                return Err(CliError::Execution(format!(
                    "job '{}' already completed (run-once); add a new job to run again",
                    spec.name
                )));
            }
            JobStatus::Active | JobStatus::Failed => {}
        }
        if let Err(breach) = spec.budgets.validate(&spec.usage) {
            return Err(CliError::Execution(format!(
                "budget exceeded for job '{}': {breach}; raise its budget limits before running again",
                spec.name
            )));
        }
        if StrategyPayload::from_metadata(spec).is_none() {
            return Err(CliError::Execution(format!(
                "job '{}' has no executable payload: add one via `vyapaar jobs add --goal <goal> --strategy <name> --symbol <instrument>`",
                spec.name
            )));
        }

        let now = Utc::now();
        self.registry()?
            .claim_job(
                &spec.id,
                WORKER_ID,
                Duration::from_secs(LEASE_TTL_SECONDS),
                now,
            )
            .map_err(into_cli)?;

        let started = Instant::now();
        let outcome = match self.dispatch_run(spec, trigger) {
            Ok(outcome) => outcome,
            // The tokio runtime failed to start before the worker ran: drop
            // the lease now instead of stranding the job claim until TTL.
            Err(dispatch_err) => {
                let _ = self.release(spec.id, false, JobStatus::Active, now);
                return Err(dispatch_err);
            }
        };
        let run_happened = outcome.run_id.is_some();

        // Composite registry status: a completed run marks once-jobs Completed;
        // a journaled failure marks once-jobs Failed; anything else rests
        // Active so the next scheduled fire retries it.
        let next_status = match outcome.disposition {
            RunDisposition::Completed if spec.once => JobStatus::Completed,
            RunDisposition::Failed if spec.once && run_happened => JobStatus::Failed,
            _ => JobStatus::Active,
        };
        let job_status = self
            .release(spec.id, run_happened, next_status, now)?
            .job_status;

        let journal_dir = outcome
            .run_id
            .as_ref()
            .map(|id| self.journals_dir().join(id.as_str()));
        Ok(RunReceipt {
            duration_ms: started.elapsed().as_millis(),
            disposition: outcome.disposition,
            run_id: outcome.run_id,
            error: outcome.error,
            job_status,
            journal_dir,
        })
    }

    /// Admits one run to the bounded worker pool and awaits its journal.
    fn dispatch_run(&self, spec: &JobSpec, trigger: &str) -> Result<JournalOutcome, CliError> {
        let pool = self.pool.clone();
        let journals = self.journals_dir();
        let spec = spec.clone();
        let trigger = trigger.to_string();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| CliError::Execution(format!("failed to start tokio: {e}")))?;
        let outcome = rt.block_on(async move {
            pool.dispatch(move || execute_in_journal(journals, spec, trigger))
                .await
        });
        match outcome {
            Ok(outcome) => Ok(outcome),
            Err(PoolError::QueueSaturated { max_queued }) => Ok(JournalOutcome {
                // Not journaled at all: no RunId was minted, so there is
                // nothing to recover for a refused admission.
                run_id: None,
                disposition: RunDisposition::Backpressured,
                error: Some(format!(
                    "worker pool queue saturated (HTTP 429 backpressure, max {max_queued} queued); retry later"
                )),
            }),
            // Panic inside the worker: no journal was opened, so there is no
            // run to recover; the job rests Active for a retry.
            Err(other) => Ok(JournalOutcome {
                run_id: None,
                disposition: RunDisposition::Failed,
                error: Some(other.to_string()),
            }),
        }
    }

    /// Releases the lease, records budget usage, and advances the cron cursor.
    ///
    /// Usage is recorded AFTER the release so budget breaches (and the
    /// `PendingApproval` they set) win over the lease transition.
    fn release(
        &self,
        job_id: uuid::Uuid,
        run_happened: bool,
        next_status: JobStatus,
        now: DateTime<Utc>,
    ) -> Result<ReleaseSummary, CliError> {
        let mut registry = self.registry()?;
        let fresh = registry
            .get_job(&job_id)
            .cloned()
            .ok_or_else(|| CliError::Execution(format!("job {job_id} vanished mid-run")))?;
        let token = fresh
            .active_lease
            .as_ref()
            .map(|lease| lease.fencing_token)
            .ok_or_else(|| CliError::Execution("no active lease to release".to_string()))?;
        registry
            .release_lease(&job_id, WORKER_ID, token, next_status, now)
            .map_err(into_cli)?;

        let mut delta = BudgetUsage::default();
        if run_happened {
            delta.record_turn(now);
            delta.record_tool_calls(1, now);
        }
        let job_status = registry
            .record_usage(&job_id, &delta, now)
            .map_err(into_cli)?;

        // Advance the schedule cursor so the next tick fires at the next
        // fire time. Manual (schedule-less) jobs have no cursor.
        let mut next_run_at = None;
        if fresh.metadata.get(TRIGGER_KEY).map(String::as_str) != Some(MANUAL_TRIGGER) {
            if let Some(job) = registry.get_job_mut(&job_id) {
                next_run_at = job.schedule.next_execution(now);
                job.next_run_at = next_run_at;
            }
            registry.save().map_err(into_cli)?;
        }
        Ok(ReleaseSummary {
            job_status,
            next_run_at,
        })
    }

    /// Evaluates schedules against `now`, holds breaches/manually-gated jobs,
    /// and dispatches what is due (`vyapaar jobs tick`).
    pub fn tick(&self) -> Result<TickReport, CliError> {
        let now = Utc::now();
        let recovered_runs = recover_interrupted_runs(&self.journals_dir())
            .map_err(|e| CliError::Execution(format!("crash recovery failed: {e}")))?;
        let mut registry = self.registry()?;
        let swept_leases = registry.sweep_expired_leases(now).len();

        let mut report = TickReport {
            recovered_runs,
            swept_leases,
            ..Default::default()
        };

        let mut due_jobs = Vec::new();
        for spec in registry.list_jobs() {
            match spec.status {
                JobStatus::Paused | JobStatus::Completed | JobStatus::Failed => {}
                JobStatus::PendingApproval => report.held.push((
                    short_id(&spec),
                    "awaiting approval (`vyapaar jobs approvals`)".to_string(),
                )),
                JobStatus::Active => {
                    if spec.metadata.get(TRIGGER_KEY).map(String::as_str) == Some(MANUAL_TRIGGER) {
                        continue;
                    }
                    if let Err(breach) = spec.budgets.validate(&spec.usage) {
                        // Record the (empty) usage so the breached job lands in
                        // the approvals queue for the next human review.
                        let _ = registry.record_usage(&spec.id, &BudgetUsage::default(), now);
                        report
                            .held
                            .push((short_id(&spec), format!("budget exceeded: {breach}")));
                        continue;
                    }
                    if spec.approval_policy == ApprovalPolicy::Manual {
                        // Hold the run behind a human gate: flipping the status
                        // through record_usage queues it for `jobs approvals`.
                        let _ = registry.record_usage(&spec.id, &BudgetUsage::default(), now);
                        report.held.push((
                            short_id(&spec),
                            "manual approval policy (`vyapaar jobs approve <id>` to allow)"
                                .to_string(),
                        ));
                        continue;
                    }
                    let anchor = spec.last_run_at.unwrap_or(spec.created_at);
                    match spec.schedule.next_execution(anchor) {
                        Some(fire) if fire <= now => due_jobs.push(spec),
                        Some(fire) => report
                            .pending_until
                            .push((short_id(&spec), fire.to_rfc3339())),
                        None => report.pending_until.push((
                            short_id(&spec),
                            "never (schedule has no future fire time)".to_string(),
                        )),
                    }
                }
            }
        }

        for spec in due_jobs {
            let receipt = match self.run(&spec, "cron") {
                Ok(receipt) => receipt,
                Err(err) => RunReceipt {
                    disposition: RunDisposition::Failed,
                    run_id: None,
                    error: Some(err.to_string()),
                    job_status: spec.status,
                    journal_dir: None,
                    duration_ms: 0,
                },
            };
            report.executed.push((short_id(&spec), receipt));
        }
        Ok(report)
    }

    /// Reads the journal trail of a job (`vyapaar jobs logs <id>`).
    ///
    /// Scans `~/.vyapaar/journals` for run dirs whose manifest carries this
    /// `job_id`, newest first (monotonic run ids sort lexicographically).
    pub fn journal_runs(&self, spec: &JobSpec) -> Result<Vec<JobRunRecord>, CliError> {
        let dir = self.journals_dir();
        let mut runs = Vec::new();
        if dir.exists() {
            for entry in std::fs::read_dir(&dir)
                .map_err(|e| CliError::Execution(format!("cannot read journals dir: {e}")))?
            {
                let entry = entry
                    .map_err(|e| CliError::Execution(format!("cannot read journals dir: {e}")))?;
                let manifest_path = entry.path().join("manifest.json");
                let Ok(manifest_content) = std::fs::read_to_string(&manifest_path) else {
                    continue;
                };
                let manifest: serde_json::Value = match serde_json::from_str(&manifest_content) {
                    Ok(value) => value,
                    Err(_) => continue,
                };
                let job_id = manifest.get("job_id").and_then(|v| v.as_str());
                let kind_matches = manifest.get("kind").and_then(|v| v.as_str()) == Some("job");
                if !kind_matches || job_id != Some(spec.id.simple().to_string().as_str()) {
                    continue;
                }
                let events = std::fs::read_to_string(entry.path().join("events.ndjson"))
                    .map(|content| {
                        content
                            .lines()
                            .filter(|line| !line.trim().is_empty())
                            .map(|line| {
                                serde_json::from_str::<serde_json::Value>(line)
                                    .unwrap_or(serde_json::json!({"raw": line}))
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                runs.push(JobRunRecord {
                    dir: entry.path(),
                    manifest,
                    events,
                });
            }
        }
        // Monotonic run ids sort lexicographically by issue order.
        runs.sort_by(|a, b| {
            let key = |record: &JobRunRecord| {
                record
                    .manifest
                    .get("run_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            };
            key(b).cmp(&key(a))
        });
        Ok(runs)
    }

    /// Pauses a job (`vyapaar jobs pause <id>`).
    pub fn pause(&self, token: &str) -> Result<JobSpec, CliError> {
        let spec = self.resolve_job(token)?;
        self.registry()?.pause_job(&spec.id).map_err(into_cli)?;
        Ok(spec)
    }

    /// Resumes a paused job (`vyapaar jobs resume <id>`).
    pub fn resume(&self, token: &str) -> Result<JobSpec, CliError> {
        let spec = self.resolve_job(token)?;
        self.registry()?.resume_job(&spec.id).map_err(into_cli)?;
        Ok(spec)
    }

    /// The human-approval queue (`vyapaar jobs approvals`).
    pub fn approvals(&self) -> Result<Vec<JobSpec>, CliError> {
        Ok(self.registry()?.approvals_queue())
    }

    /// Approves a held job, returning it to `Active`
    /// (`vyapaar jobs approve <id>`).
    pub fn approve(&self, token: &str) -> Result<JobSpec, CliError> {
        let spec = self.resolve_job(token)?;
        self.registry()?.approve_job(&spec.id).map_err(into_cli)?;
        Ok(spec)
    }

    /// Denies a held job (`vyapaar jobs deny <id>`): the proposal is rejected
    /// and the job rests `Paused` (recoverable via `resume` after budgets or
    /// strategies are fixed; the registry has no terminal-reject state).
    pub fn deny(&self, token: &str) -> Result<JobSpec, CliError> {
        let spec = self.resolve_job(token)?;
        let mut registry = self.registry()?;
        match registry.get_job_mut(&spec.id) {
            Some(job) => {
                if job.status != JobStatus::PendingApproval {
                    return Err(CliError::Execution(format!(
                        "job '{}' is not awaiting approval (current status: {:?})",
                        job.name, job.status
                    )));
                }
                job.status = JobStatus::Paused;
            }
            None => return Err(CliError::Execution(format!("job '{token}' not found"))),
        }
        registry.save().map_err(into_cli)?;
        Ok(spec)
    }
}

/// One journaled run of a job (manifest + event stream).
#[derive(Debug, Clone)]
pub struct JobRunRecord {
    pub dir: PathBuf,
    pub manifest: serde_json::Value,
    pub events: Vec<serde_json::Value>,
}

struct ReleaseSummary {
    job_status: JobStatus,
    #[allow(dead_code)]
    next_run_at: Option<DateTime<Utc>>,
}

/// What the in-journal worker reported back to the dispatcher.
struct JournalOutcome {
    /// The minted run id (`None` when the run was refused before minting).
    run_id: Option<RunId>,
    disposition: RunDisposition,
    error: Option<String>,
}

/// Applies the ADR 0017 runs state machine to derive a manifest snapshot.
fn status_after_event(
    from: RunStatus,
    event: RunEvent,
    what: &'static str,
) -> Result<RunStatus, CliError> {
    from.next(event)
        .map_err(|e| CliError::Execution(format!("{what}: {e}")))
}

/// Serializes a failure into the shared wire `ErrorDetail` JSON shape
/// ("code"/"message"/"retryable"/"context"), the same shape the crash sweeper
/// patches interrupted manifests with.
pub fn error_detail_value(message: impl std::fmt::Display) -> serde_json::Value {
    let msg = message.to_string();
    serde_json::to_value(
        ErrorDetail::new(ErrorCode::InternalError, msg.clone())
            .with_context(serde_json::json!({"reason": "job_run"})),
    )
    .unwrap_or_else(|_| {
        serde_json::json!({
            "code": "internal_error",
            "message": msg,
            "retryable": false,
            "context": {"reason": "job_run"}
        })
    })
}

/// Mints a fresh globally-monotonic run id (Crockford Base32 ULID, ADR 0017).
/// Entropy comes from a v4 UUID's first 10 bytes (80 random bits).
pub fn mint_run_id() -> Result<RunId, CliError> {
    let unix_ms = u64::try_from(Utc::now().timestamp_millis())
        .map_err(|_| CliError::Execution("wall clock before the Unix epoch".to_string()))?;
    let mut entropy = [0u8; 10];
    entropy.copy_from_slice(&uuid::Uuid::new_v4().as_bytes()[..10]);
    RunIdGenerator::default()
        .next(unix_ms, entropy)
        .map_err(|e| CliError::Execution(format!("run id generation failed: {e}")))
}

/// Short (8-hex) display id for a job.
pub fn short_id(spec: &JobSpec) -> String {
    spec.id.simple().to_string()[..8].to_string()
}

/// Human summary of one executed run for CLI text output.
pub fn summarize_receipt(receipt: &RunReceipt) -> String {
    match receipt.disposition {
        RunDisposition::Backpressured => format!(
            "run refused: {}",
            receipt
                .error
                .clone()
                .unwrap_or_else(|| "backpressure".into())
        ),
        RunDisposition::Failed => format!(
            "run failed: {} (see `vyapaar jobs logs` / journal {})",
            receipt
                .error
                .clone()
                .unwrap_or_else(|| "unknown error".into()),
            receipt
                .journal_dir
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "n/a".into())
        ),
        RunDisposition::Completed => {
            let journal = receipt
                .journal_dir
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            format!("run completed; journal at {journal}")
        }
    }
}

/// Builds [`JobBudgets`] from an [`AddRequest`].
fn budgets_from_request(req: &AddRequest) -> JobBudgets {
    let mut budgets = JobBudgets::new().with_max_turns(req.max_turns);
    if let Some(inr) = req.max_inr {
        budgets = budgets.with_max_inr(inr);
    }
    if let Some(usd) = req.max_usd {
        budgets = budgets.with_max_usd(usd);
    }
    if let Some(tokens) = req.max_tokens {
        budgets = budgets.with_max_tokens(tokens);
    }
    if let Some(tool_calls) = req.max_tool_calls {
        budgets = budgets.with_max_tool_calls(tool_calls);
    }
    if let Some(data_calls) = req.max_data_calls {
        budgets = budgets.with_max_data_calls(data_calls);
    }
    budgets
}

/// Maps any registry/pool/displayed error into the CLI error surface.
fn into_cli(err: impl std::fmt::Display) -> CliError {
    CliError::Execution(err.to_string())
}

/// The job-shaped journal manifest value shared by all snapshots.
#[allow(clippy::too_many_arguments)]
fn job_manifest(
    spec: &JobSpec,
    run_id: &RunId,
    status: RunStatus,
    trigger: &str,
    created_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    finished_at: Option<DateTime<Utc>>,
    error: Option<serde_json::Value>,
) -> serde_json::Value {
    let payload = StrategyPayload::from_metadata(spec);
    let mut manifest = serde_json::json!({
        "manifest_version": 1,
        "kind": "job",
        "run_id": run_id.as_str(),
        "job_id": spec.id.simple().to_string(),
        "job_name": spec.name,
        "goal": spec.goal,
        "trigger": trigger,
        "status": status,
        "read_only": spec.read_only,
        "once": spec.once,
        "created_at": created_at.to_rfc3339(),
    });
    if let Some(at) = started_at {
        manifest["started_at"] = serde_json::json!(at.to_rfc3339());
    }
    if let Some(at) = finished_at {
        manifest["finished_at"] = serde_json::json!(at.to_rfc3339());
    }
    if let Some(err) = error {
        manifest["error"] = err;
    }
    if let Some(payload) = payload {
        manifest["strategy"] = serde_json::json!(payload.strategy);
        manifest["symbol"] = serde_json::json!(payload.symbol);
        manifest["capital"] = serde_json::json!(payload.capital.to_string());
        if let Some(params) = &payload.params {
            // Stringified params keep the manifest replayable across Decimal
            // serde feature flags (zero-float policy held at the boundary).
            if let Ok(json) = serde_json::to_value(
                params
                    .iter()
                    .map(|(k, v)| (k.clone(), v.to_string()))
                    .collect::<BTreeMap<String, String>>(),
            ) {
                manifest["params"] = json;
            }
        }
    }
    manifest
}

/// Serialized by the dispatcher onto the bounded pool's blocking threads.
///
/// Journal contract (deterministic payloads, timestamps only in the manifest):
/// 1. `run.queued`   — admission snapshot (`Pending`).
/// 2. `run.started`  — manifest flips to `Running` via `write_manifest`; the
///    crash sweeper treats that shape as interrupted.
/// 3. `run.result`   — the full engine report (or `run.failed`).
/// 4. `run.finalized` — journal files flushed (gravity of finalize).
fn execute_in_journal(journals_dir: PathBuf, spec: JobSpec, trigger: String) -> JournalOutcome {
    let Some(payload) = StrategyPayload::from_metadata(&spec) else {
        return JournalOutcome {
            run_id: None,
            disposition: RunDisposition::Failed,
            error: Some("job has no executable payload (strategy metadata missing)".into()),
        };
    };

    let created_at = Utc::now();
    let run_id = match mint_run_id() {
        Ok(run_id) => run_id,
        Err(err) => {
            return JournalOutcome {
                run_id: None,
                disposition: RunDisposition::Failed,
                error: Some(err.to_string()),
            };
        }
    };

    let mut writer = match FileJournalWriter::create_from_value(
        &journals_dir,
        &run_id,
        &job_manifest(
            &spec,
            &run_id,
            RunStatus::Pending,
            &trigger,
            created_at,
            None,
            None,
            None,
        ),
    ) {
        Ok(writer) => writer,
        Err(e) => {
            return JournalOutcome {
                run_id: Some(run_id),
                disposition: RunDisposition::Failed,
                error: Some(format!("journal write failed: {e}")),
            };
        }
    };

    // event 1 — queued
    let queued = writer.append_event(&serde_json::json!({
        "seq": 1,
        "type": "run.queued",
        "trigger": trigger,
        "job_id": spec.id.simple().to_string(),
    }));

    let running = status_after_event(RunStatus::Pending, RunEvent::Start, "run admission");
    let running_status = match running {
        Ok(status) => status,
        Err(err) => return JournalOutcome::failed(Some(run_id), err),
    };
    // event 2 — started (manifest snapshot flips pending -> running)
    let started = queued.and_then(|()| {
        writer
            .write_manifest(&job_manifest(
                &spec,
                &run_id,
                running_status,
                &trigger,
                created_at,
                Some(created_at),
                None,
                None,
            ))
            .and_then(|()| {
                writer.append_event(&serde_json::json!({
                    "seq": 2,
                    "type": "run.started",
                    "worker": WORKER_ID,
                }))
            })
    });
    if started.is_err() {
        return JournalOutcome::io_failed(run_id);
    }

    // event 3 — the deterministic engine execution
    let eng_result = StrategyBridge::run_backtest(
        &payload.strategy,
        &payload.symbol,
        payload.params.as_ref(),
        payload.capital,
    );

    match eng_result {
        Ok(report) => {
            let completed =
                status_after_event(running_status, RunEvent::Complete, "run completion");
            let completed_status = match completed {
                Ok(status) => status,
                Err(err) => return JournalOutcome::failed(Some(run_id), err),
            };
            let report_value = serde_json::to_value(&report).unwrap_or_default();
            let _ = writer.append_event(&serde_json::json!({
                "seq": 3,
                "type": "run.result",
                "strategy": payload.strategy,
                "symbol": payload.symbol,
                "report": report_value,
            }));
            let _ = writer.append_event(&serde_json::json!({
                "seq": 4,
                "type": "run.finalized",
                "status": "completed",
            }));
            let finished_at = Utc::now();
            let final_manifest = job_manifest(
                &spec,
                &run_id,
                completed_status,
                &trigger,
                created_at,
                Some(created_at),
                Some(finished_at),
                None,
            );
            // Gap 0002 closed: `BacktestReport` now carries per-trade rows, so
            // `trades.json` journals the serialized row list (replayable
            // tax-lot evidence) instead of only the aggregate count. Rows are
            // kept `Decimal`-only — zero floats.
            let trades = serde_json::to_value(&report.trades).unwrap_or_default();
            if writer
                .finalize_value(&final_manifest, &trades, &report_value)
                .is_err()
            {
                return JournalOutcome::io_failed(run_id);
            }
            JournalOutcome {
                run_id: Some(run_id),
                disposition: RunDisposition::Completed,
                error: None,
            }
        }
        Err(eng_err) => {
            let failed_status = status_after_event(running_status, RunEvent::Fail, "run failure");
            let failed_status = match failed_status {
                Ok(status) => status,
                Err(err) => return JournalOutcome::failed(Some(run_id), err),
            };
            let _ = writer.append_event(&serde_json::json!({
                "seq": 3,
                "type": "run.failed",
                "error": eng_err.clone(),
            }));
            let _ = writer.append_event(&serde_json::json!({
                "seq": 4,
                "type": "run.finalized",
                "status": "failed",
            }));
            let final_manifest = job_manifest(
                &spec,
                &run_id,
                failed_status,
                &trigger,
                created_at,
                Some(created_at),
                Some(Utc::now()),
                Some(error_detail_value(format!(
                    "strategy engine error: {eng_err}"
                ))),
            );
            let _ = writer.finalize_value(
                &final_manifest,
                &serde_json::json!({}),
                &serde_json::json!({}),
            );
            JournalOutcome {
                run_id: Some(run_id),
                disposition: RunDisposition::Failed,
                error: Some(eng_err),
            }
        }
    }
}

impl JournalOutcome {
    /// The journal stopped mid-write: report failed; the crash sweeper
    /// normalizes whatever snapshot persisted.
    fn io_failed(run_id: RunId) -> Self {
        Self {
            run_id: Some(run_id),
            disposition: RunDisposition::Failed,
            error: Some(
                "journal write failed mid-run (state recovered by the crash sweeper)".into(),
            ),
        }
    }

    /// Engine-internal failure without a journal surface.
    fn failed(run_id: Option<RunId>, err: CliError) -> Self {
        Self {
            run_id,
            disposition: RunDisposition::Failed,
            error: Some(err.to_string()),
        }
    }
}

/// Sweeps interrupted run journals (`Pending`/`Running` snapshots -> `Failed`)
/// at process boot. Missing directories count as zero work; I/O failures are
/// non-fatal (the next `vyapaar jobs tick` sweeps again).
pub fn boot_recover(home: Option<&Path>) -> usize {
    let Some(home) = home else {
        return 0;
    };
    recover_interrupted_runs(&home.join("journals")).unwrap_or(0)
}
