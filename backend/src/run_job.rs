//! `cargo doido jobs run|list` — perform jobs in-process (until doido-jobs ships this natively).
//! See repo `RUN_JOB.md` for the upstream spec.

use std::sync::Arc;
use std::time::Duration;

use doido_jobs::config::{build_configured_queue, load as load_jobs_config};
use doido_jobs::{JobContext, JobPayload, JobQueue, JobRegistry};

#[derive(Debug, Default)]
struct RunArgs {
    job_name: String,
    payload: serde_json::Value,
    queue: Option<String>,
    timeout_secs: Option<u64>,
    later: bool,
}

pub fn is_jobs_run_cli() -> bool {
    matches!(
        (std::env::args().nth(1).as_deref(), std::env::args().nth(2).as_deref()),
        (Some("jobs"), Some("run" | "list"))
    )
}

pub async fn execute_from_env() -> i32 {
    match std::env::args().nth(2).as_deref() {
        Some("list") => run_list(),
        Some("run") => match parse_run_args(std::env::args().skip(3)) {
            Ok(args) => run_job(args).await,
            Err(msg) => {
                eprintln!("{msg}");
                eprintln!("uso: cargo doido jobs run <job_name> [--payload '{{}}'] [--queue catalog] [--timeout 3600] [--later]");
                1
            }
        },
        _ => 1,
    }
}

fn run_list() -> i32 {
    let registry = JobRegistry::from_inventory();
    let mut names = registry.names();
    names.sort_unstable();
    if names.is_empty() {
        eprintln!("nenhum job #[job] registrado neste binário");
        return 0;
    }
    for n in names {
        println!("{n}");
    }
    0
}

fn parse_run_args<I>(args: I) -> Result<RunArgs, String>
where
    I: IntoIterator<Item = String>,
{
    let mut iter = args.into_iter();
    let job_name = iter
        .next()
        .filter(|s| !s.starts_with('-'))
        .ok_or_else(|| "informe o nome do job (ex.: sync_municipio_cna_lote)".to_string())?;

    let mut out = RunArgs {
        job_name,
        payload: serde_json::json!({}),
        ..Default::default()
    };

    let mut tail: Vec<String> = iter.collect();
    let mut i = 0;
    while i < tail.len() {
        match tail[i].as_str() {
            "--payload" => {
                i += 1;
                let raw = tail
                    .get(i)
                    .ok_or_else(|| "--payload requer um JSON".to_string())?;
                out.payload = serde_json::from_str(raw)
                    .map_err(|e| format!("--payload JSON inválido: {e}"))?;
            }
            "--queue" => {
                i += 1;
                out.queue = Some(
                    tail.get(i)
                        .ok_or_else(|| "--queue requer um nome".to_string())?
                        .clone(),
                );
            }
            "--timeout" => {
                i += 1;
                let raw = tail
                    .get(i)
                    .ok_or_else(|| "--timeout requer segundos".to_string())?;
                out.timeout_secs = Some(
                    raw.parse()
                        .map_err(|_| format!("--timeout inválido: {raw}"))?,
                );
            }
            "--later" => out.later = true,
            other => return Err(format!("opção desconhecida: {other}")),
        }
        i += 1;
    }

    Ok(out)
}

fn default_queue(job_name: &str) -> &'static str {
    match job_name {
        "import_nfse_anexo_a" | "assinar_documento" | "transmitir_documento" => "default",
        "sync_nfse_catalog_daily"
        | "sync_municipio_cna_coordinator"
        | "sync_municipio_cna_lote" => "catalog",
        _ => "default",
    }
}

fn default_timeout(job_name: &str) -> u64 {
    match job_name {
        "sync_nfse_catalog_daily" => 3600,
        "sync_municipio_cna_coordinator" => 600,
        "sync_municipio_cna_lote" => 1800,
        "import_nfse_anexo_a" => 900,
        _ => 3600,
    }
}

fn build_payload(args: &RunArgs) -> JobPayload {
    let queue = args
        .queue
        .clone()
        .unwrap_or_else(|| default_queue(&args.job_name).to_string());
    let timeout = args.timeout_secs.unwrap_or_else(|| default_timeout(&args.job_name));
    JobPayload::new(queue, args.payload.clone(), 0)
        .with_name(&args.job_name)
        .with_timeout(timeout)
}

async fn run_job(args: RunArgs) -> i32 {
    let registry = JobRegistry::from_inventory();
    if !registry
        .names()
        .iter()
        .any(|n| *n == args.job_name.as_str())
    {
        eprintln!("job `{}` não registrado neste binário", args.job_name);
        eprintln!("jobs disponíveis (cargo doido jobs list):");
        for n in registry.names() {
            eprintln!("  {n}");
        }
        return 1;
    }

    let job = build_payload(&args);

    if args.later {
        return enqueue_job(job).await;
    }

    perform_now(job, &registry).await
}

async fn enqueue_job(job: JobPayload) -> i32 {
    let config = load_jobs_config();
    let queue = match build_configured_queue(&config).await {
        Ok(q) => q,
        Err(e) => {
            eprintln!("fila de jobs: {e}");
            return 1;
        }
    };
    let id = job.id.clone();
    let name = job.job_name.clone();
    let qname = job.queue.clone();
    match queue.enqueue(job).await {
        Ok(_) => {
            println!("enqueued job_id={id} queue={qname} job_name={name}");
            eprintln!("processe com: cargo doido worker --once");
            0
        }
        Err(e) => {
            eprintln!("enqueue falhou: {e}");
            1
        }
    }
}

async fn perform_now(job: JobPayload, registry: &JobRegistry) -> i32 {
    let timeout = Duration::from_secs(job.timeout);
    let ctx = Arc::new(JobContext::new());
    let name = job.job_name.clone();
    eprintln!("executando job `{name}` (timeout {}s)…", job.timeout);

    match tokio::time::timeout(timeout, registry.dispatch(job, ctx)).await {
        Ok(Ok(())) => {
            eprintln!("job `{name}` concluído");
            0
        }
        Ok(Err(e)) => {
            eprintln!("job `{name}` falhou: {e}");
            1
        }
        Err(_) => {
            eprintln!("job `{name}` excedeu timeout de {}s", timeout.as_secs());
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_run_minimal() {
        let a = parse_run_args(["sync_municipio_cna_lote".to_string()].into_iter()).unwrap();
        assert_eq!(a.job_name, "sync_municipio_cna_lote");
        assert!(!a.later);
    }

    #[test]
    fn parse_run_payload() {
        let a = parse_run_args(
            [
                "sync_municipio_cna_lote".into(),
                "--payload".into(),
                r#"{"ibge_codes":["3304557"]}"#.into(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(a.payload["ibge_codes"][0], "3304557");
    }

    #[test]
    fn default_queue_catalog() {
        assert_eq!(default_queue("sync_nfse_catalog_daily"), "catalog");
        assert_eq!(default_queue("import_nfse_anexo_a"), "default");
    }
}
