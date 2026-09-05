//! Run the external operator verifier against a fast, deterministic API server double.
use std::{fs, os::unix::fs::PermissionsExt, process::Command};

const KUBECTL: &str = r##"#!/usr/bin/env python3
import json, os, sys
from pathlib import Path
args = sys.argv[1:]
state_path = Path(os.environ['MOCK_STATE'])
state = json.loads(state_path.read_text()) if state_path.exists() else {'patches': 0, 'deleted': False, 'reads': 0, 'read_counts': {}}
def save(): state_path.write_text(json.dumps(state))
app = os.environ['MOCK_APP']
name = 'sift-store' if app == 'sift' else app
desired = 3 if app == 'sift' else 1
def workload(replicas, generation, observed=None):
    return {'metadata': {'name': name, 'namespace': app, 'uid': 'same-workload', 'generation': generation},
            'spec': {'replicas': replicas},
            'status': {'readyReplicas': desired, 'observedGeneration': generation if observed is None else observed}}
if args[:2] == ['auth', 'can-i']: print('yes')
elif 'patch' in args:
    state['patches'] += 1
    state['reads'] = 0
    save()
    generation = state['patches'] * 2
    if '-o' in args:
        replicas = desired if os.environ.get('MOCK_BAD_PATCH') == '1' else 0
        print(json.dumps(workload(replicas, generation)))
    else: print('statefulset.apps/' + name + ' patched')
elif 'get' in args and ('statefulset/' + name) in args:
    state['reads'] += 1
    state['read_counts'][str(state['patches'])] = state['reads']
    save()
    # The controller has already repaired the write before the next GET.
    generation = state['patches'] * 2 + 1
    if 'json' in args:
        result = workload(desired, generation)
        if state['reads'] == 1:
            mode = os.environ['MOCK_GET_MODE']
            if mode == 'changed_uid': result['metadata']['uid'] = 'replacement-workload'
            elif mode == 'same_generation':
                result['metadata']['generation'] = generation - 1
                result['status']['observedGeneration'] = generation - 1
            else: result['status']['observedGeneration'] = generation - 1
        print(json.dumps(result))
    else: print(desired)
elif 'get' in args and 'statefulset/sift-control' in args:
    result = workload(3, 1)
    result['metadata']['name'] = 'sift-control'
    print(json.dumps(result))
elif 'get' in args and any(a.startswith('lease/') for a in args):
    print('new-leader' if state['deleted'] else 'old-leader')
elif 'get' in args and any(a.startswith('pod/') for a in args): print('pod exists')
elif 'delete' in args:
    state['deleted'] = True
    save()
elif 'scale' in args or 'rollout' in args: pass
else: raise SystemExit('unexpected kubectl: ' + repr(args))
"##;

fn verify(app: &str, bad_patch: bool, get_mode: &str) -> (std::process::Output, tempfile::TempDir) {
    let temp = tempfile::tempdir().unwrap();
    for (name, contents) in [("kubectl", KUBECTL), ("sleep", "#!/bin/sh\nexit 0\n")] {
        let path = temp.path().join(name);
        fs::write(&path, contents).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new("bash")
        .arg(root.join("acceptance/gcp/scripts/verify-operator-cell.sh"))
        .arg(app)
        .env(
            "PATH",
            format!(
                "{}:{}",
                temp.path().display(),
                std::env::var("PATH").unwrap()
            ),
        )
        .env("EVIDENCE_DIR", temp.path().join("evidence"))
        .env("MOCK_STATE", temp.path().join("state.json"))
        .env("MOCK_APP", app)
        .env("MOCK_BAD_PATCH", if bad_patch { "1" } else { "0" })
        .env("MOCK_GET_MODE", get_mode)
        .output()
        .unwrap();
    (output, temp)
}

#[test]
fn fast_reconcile_passes_before_and_after_leader_takeover() {
    for app in ["sift", "tape"] {
        let (output, temp) = verify(app, false, "stale_observed_generation");
        assert!(
            output.status.success(),
            "{app}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let state: serde_json::Value =
            serde_json::from_slice(&fs::read(temp.path().join("state.json")).unwrap()).unwrap();
        assert_eq!(state["patches"], 2);
        assert_eq!(state["deleted"], true);
        assert!(
            ["1", "2"]
                .iter()
                .all(|patch| state["read_counts"][patch].as_u64().unwrap() >= 2),
            "must wait for observed generation"
        );
        assert!(temp
            .path()
            .join(format!("evidence/{app}-operator-cell.json"))
            .exists());
    }
}

#[test]
fn patch_receipt_must_prove_that_drift_was_applied() {
    let (output, temp) = verify("sift", true, "stale_observed_generation");
    assert!(!output.status.success());
    assert!(!temp
        .path()
        .join("evidence/sift-operator-cell.json")
        .exists());
}

#[test]
fn reconcile_waits_for_the_same_object_and_a_newer_generation() {
    for mode in ["changed_uid", "same_generation"] {
        let (output, temp) = verify("sift", false, mode);
        assert!(
            output.status.success(),
            "{mode}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let state: serde_json::Value =
            serde_json::from_slice(&fs::read(temp.path().join("state.json")).unwrap()).unwrap();
        assert_eq!(state["patches"], 2);
        for patch in ["1", "2"] {
            assert!(
                state["read_counts"][patch].as_u64().unwrap() >= 2,
                "{mode}: must not accept the invalid first GET after patch {patch}"
            );
        }
        assert!(temp
            .path()
            .join("evidence/sift-operator-cell.json")
            .exists());
    }
}
