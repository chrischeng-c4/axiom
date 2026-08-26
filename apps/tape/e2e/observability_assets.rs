// HANDWRITE-BEGIN gap="missing-generator:unit-test:66bdfe4f" tracker="#1588" reason="Parse the monitoring manifests and assert scrape target labels and metric-series references. generator gap: missing-generator:observability-asset-test (#1588)."
//! Offline contract checks for the optional Prometheus Operator component.

use serde_yaml::Value;

fn yaml(path: &str) -> Value {
    serde_yaml::from_str(path).expect("observability manifest parses")
}

#[test]
fn servicemonitor_scrapes_metrics_and_preserves_service_labels() {
    let doc = yaml(include_str!(
        "../k8s/components/observability/servicemonitor.yaml"
    ));
    assert_eq!(doc["kind"], "ServiceMonitor");
    assert_eq!(doc["spec"]["endpoints"][0]["path"], "/metrics");
    assert_eq!(doc["spec"]["targetLabels"][0], "app");
    assert_eq!(doc["spec"]["targetLabels"][1], "role");
}

#[test]
fn alert_rules_only_reference_existing_tape_latency_series() {
    let rule = include_str!("../k8s/components/observability/prometheusrule.yaml");
    let doc = yaml(rule);
    assert_eq!(doc["kind"], "PrometheusRule");
    for metric in [
        "tape_append_latency_ms_sum",
        "tape_append_latency_ms_count",
        "tape_replay_latency_ms_sum",
        "tape_replay_latency_ms_count",
        // #2573 — the degraded-mode gauge `TapeMetrics::render` publishes.
        "tape_storage_degraded",
    ] {
        assert!(rule.contains(metric), "missing real Tape metric {metric}");
    }
}

/// #3051 — memory headroom alert references cAdvisor series, not tape-scraped
/// metrics. cAdvisor series are published by kubelet and carried by kube-state-
/// metrics / Prometheus Operator's default scrape config; they are not `tape_*`
/// names so this is a separate pinned assertion.
#[test]
fn alert_rules_reference_cadvisor_memory_series() {
    let rule = include_str!("../k8s/components/observability/prometheusrule.yaml");
    for metric in [
        "container_memory_working_set_bytes",
        "container_spec_memory_limit_bytes",
    ] {
        assert!(
            rule.contains(metric),
            "TapeMemoryHeadroomLow must reference cAdvisor series {metric}"
        );
    }
}
// HANDWRITE-END
