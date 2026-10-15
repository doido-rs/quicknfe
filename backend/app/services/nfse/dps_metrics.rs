//! Métricas de cobertura DPS (XML emitido × leiaute × presença mapeada).

use std::path::Path;

use super::dps::coverage;
use super::dps_presenca;

#[derive(Debug, Clone)]
pub struct DpsMetricsReport {
    pub mandatory_total: usize,
    pub emitted_hits: usize,
    pub emitted_ratio: f64,
    pub presence_mapped_count: usize,
    pub presence_mapped_on_mandatory: usize,
    pub missing_emitted: Vec<String>,
}

pub fn metrics_report(repo_root: &Path) -> Result<DpsMetricsReport, String> {
    let (hit, total, missing) = coverage::coverage_report(repo_root)?;
    let mapped = dps_presenca::campos_presenca_mapeados();
    let emitted = coverage::emitted_tags();
    let presence_on_mandatory = mapped.intersection(&emitted).count();
    Ok(DpsMetricsReport {
        mandatory_total: total,
        emitted_hits: hit,
        emitted_ratio: if total == 0 {
            0.0
        } else {
            hit as f64 / total as f64
        },
        presence_mapped_count: mapped.len(),
        presence_mapped_on_mandatory: presence_on_mandatory,
        missing_emitted: missing,
    })
}

pub fn format_report(report: &DpsMetricsReport) -> String {
    let mut out = format!(
        "DPS infDPS mandatory (leiaute): {}\n\
         XML emitted coverage: {}/{} ({:.1}%)\n\
         Presence map entries: {} ({} also in emitted_tags)\n",
        report.mandatory_total,
        report.emitted_hits,
        report.mandatory_total,
        report.emitted_ratio * 100.0,
        report.presence_mapped_count,
        report.presence_mapped_on_mandatory,
    );
    if !report.missing_emitted.is_empty() {
        out.push_str("\nTop missing mandatory tags (not in emitted_tags):\n");
        for line in report.missing_emitted.iter().take(25) {
            out.push_str("  - ");
            out.push_str(line);
            out.push('\n');
        }
        if report.missing_emitted.len() > 25 {
            out.push_str(&format!(
                "  … and {} more\n",
                report.missing_emitted.len() - 25
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn metrics_report_runs() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let report = metrics_report(&root).expect("metrics");
        assert!(report.mandatory_total > 0);
        assert!(report.emitted_hits > 0);
    }

    #[test]
    fn metrics_print_smoke() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let report = metrics_report(&root).unwrap();
        let text = format_report(&report);
        println!("{text}");
        assert!(text.contains("XML emitted coverage"));
    }
}
