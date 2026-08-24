#!/usr/bin/env python3
import hashlib
import json
import os
import sys

REPO_ROOT = "/Users/chrischeng/axiom/project-mamba"

PLATFORM = "aarch64-apple-darwin"
MAMBA_GIT_SHA = "03e8c5216126fa1fd2fed4447a207cb53ee62414"
MAMBA_BINARY_SHA256 = "31140775715679873318eec3a687e816b53918170b27d69474d10811c3a10c1e"
CPYTHON312_EXECUTABLE = "/Users/chrischeng/.pyenv/versions/3.12.11/bin/python3.12"
CPYTHON312_VERSION = "Python 3.12.11"
GENERATED_AT = "2026-07-30T00:00:00Z"
VERIFIER_COMMAND = "cargo test -p mamba --release --test mamba_core_semantics_ec -- force_typed_contract_completion --exact"

FAMILIES = [
    "local_binding",
    "global_binding",
    "class_attribute",
    "parameter",
    "return",
    "comprehension",
    "expression_join",
]

EXPECTED_PAIRED_SHA = {
    "local_binding": "eca2c751ff862a202825ca1075b239376a366b0cc267e831a1d75a62cbe12f70",
    "global_binding": "413168f410fed2dccabdfe31db9a60a726c7609f9ec87480845101b7bebdaecf",
    "class_attribute": "192f223d5dfcb10d0f996e6ed26931517b816e021d228c38aaf10485c04c77f2",
    "parameter": "6583974a59583855392d8f2be80bc1d8986c1bfa8dd24c4aafb3e5ec4af5a2d2",
    "return": "a0f48b448e3301ba3196574e7ad50a9e98f68fe5ec2d5dd3d0ce5692fbeff3d3",
    "comprehension": "a525da19fd97f167e223ed2c0123a99605527da7ca254a2e93ed6eacc5e9ac13",
    "expression_join": "c452583f7f4b1e2d60059ea17180bcf24e798ed743a45fae38c3d60ca5cdbefd",
}

EXPECTED_IMPLICIT_SHA = {
    "local_binding": "24bbc3d05ac998768e833af0df70964eaa4bcd52eb278e24013094764f11d464",
    "global_binding": "1250bfd953a621272864d76796e0144f269c1ae79ecc569b5749f989418ed50f",
    "class_attribute": "3888ca5169cae9780f88fe68bcf38d2655eda68815b1b5df5d58b9272aaa95e4",
    "parameter": "5bbf06ab53edc52b97313b550b3e6dd61e2201fe1bb5c26c1cefc4b83d3e52ae",
    "return": "e88780656b95151f136917f9cb305d834f9bf9baea54cda44d313642e4bec200",
    "comprehension": "e2455d2d660c91efdf2106f02ae00b96dc578562a80130d78c1045191ee819e2",
    "expression_join": "7f4470d85bd9897d5efb664a383b1837ce0873dff217a6dd74edf456e1e47b90",
}

EXPECTED_EXPLICIT_SHA = {
    "local_binding": "696296f62cf9c28bf85a693772976b645b1d50b896bba9ab8b484609a12b3953",
    "global_binding": "c7e5ffe688da4bab56939b061f5a6cc1aeea4f5ec9be9041b93f7507bc688b99",
    "class_attribute": "593d481201337dccd2a27b38053056cc20f4f659c676a1b94d4de49cd4d1fed8",
    "parameter": "8ac7dde6901dcfa17c38576177010645be7291b6af417150e66fb5ddbe47e2df",
    "return": "43cf90e0f4b5b64a34de42edb189c4841b93e362bcd70bfbfa463c8e2c5c85a6",
    "comprehension": "d771212ff506b7e130ce5191c39eb34a1645070b2313acbc42aa7a55cab382e3",
    "expression_join": "ab9e0fb5ce6b06f8e747a57d87a6b4708dc8f538185353c3d2389653cadf137b",
}

IMPLICIT_MEASURED = {
    "local_binding": {
        "source_span": "145..150",
        "call_site_or_binding_site": "8:5",
        "binding_identity": "items",
        "inference_path_classification": "local_binding -> list_literal -> element",
        "expected_probe_anchor": "cannot infer type for local binding `items`: local_binding -> list_literal -> element",
    },
    "global_binding": {
        "source_span": "104..109",
        "call_site_or_binding_site": "7:1",
        "binding_identity": "items",
        "inference_path_classification": "global_binding -> list_literal -> element",
        "expected_probe_anchor": "cannot infer type for global binding `items`: global_binding -> list_literal -> element",
    },
    "class_attribute": {
        "source_span": "123..128",
        "call_site_or_binding_site": "8:5",
        "binding_identity": "items",
        "inference_path_classification": "class_attribute -> list_literal -> element",
        "expected_probe_anchor": "cannot infer type for class attribute `items`: class_attribute -> list_literal -> element",
    },
    "parameter": {
        "source_span": "111..121",
        "call_site_or_binding_site": "7:13",
        "binding_identity": "items",
        "inference_path_classification": "parameter -> default -> list_literal -> element",
        "expected_probe_anchor": "cannot infer type for parameter `items`: parameter -> default -> list_literal -> element",
    },
    "return": {
        "source_span": "133..135",
        "call_site_or_binding_site": "8:12",
        "binding_identity": "collect",
        "inference_path_classification": "return -> list_literal -> element",
        "expected_probe_anchor": "cannot infer return type for function `collect`: return -> list_literal -> element",
    },
    "comprehension": {
        "source_span": "153..158",
        "call_site_or_binding_site": "8:5",
        "binding_identity": "items",
        "inference_path_classification": "comprehension -> generator -> iterable -> list_literal -> element",
        "expected_probe_anchor": "cannot infer comprehension type for binding `items`: comprehension -> generator -> iterable -> list_literal -> element",
    },
    "expression_join": {
        "source_span": "167..172",
        "call_site_or_binding_site": "8:5",
        "binding_identity": "items",
        "inference_path_classification": "expression_join -> branch -> list_literal -> element",
        "expected_probe_anchor": "cannot infer expression join type for binding `items`: expression_join -> branch -> list_literal -> element",
    },
}


def compute_sha256_bytes(b: bytes) -> str:
    return hashlib.sha256(b).hexdigest()


def normalize_implicit(raw_bytes: bytes) -> bytes:
    lines = raw_bytes.splitlines(keepends=True)
    idx = 0
    while idx < len(lines) and lines[idx].startswith(b"# "):
        idx += 1
    return b"".join(lines[idx:])


def normalize_explicit(raw_bytes: bytes) -> bytes:
    lines = raw_bytes.splitlines(keepends=True)
    idx = 0
    while idx < len(lines) and lines[idx].startswith(b"# "):
        idx += 1
    body = b"".join(lines[idx:])
    body = body.replace(b": Any = ", b" = ")
    body = body.replace(b" -> Any:", b":")
    return body


def main():
    gate_dir = os.path.join(
        REPO_ROOT,
        "apps/mamba/tests/governance/gates/t1_implicit_any_ingress_matrix",
    )
    evidence_dir = os.path.join(
        REPO_ROOT, "apps/mamba/external-contracts/evidence"
    )

    cases_path = os.path.join(gate_dir, "cases.jsonl")
    manifest_path = os.path.join(gate_dir, "manifest.toml")
    lock_path = os.path.join(
        evidence_dir, "mamba-t1-implicit-any-ingress-matrix-lock.json"
    )

    pairs_data = []
    fixture_records = []
    cases_rows = []

    for family in FAMILIES:
        imp_rel = f"apps/mamba/tests/cpython/_regression/core/typecheck/implicit_any_ingress/{family}.py"
        exp_rel = f"apps/mamba/tests/cpython/_regression/core/typecheck/explicit_any_acceptance/{family}.py"

        imp_abs = os.path.join(REPO_ROOT, imp_rel)
        exp_abs = os.path.join(REPO_ROOT, exp_rel)

        with open(imp_abs, "rb") as f:
            imp_bytes = f.read()
        with open(exp_abs, "rb") as f:
            exp_bytes = f.read()

        imp_sha = compute_sha256_bytes(imp_bytes)
        exp_sha = compute_sha256_bytes(exp_bytes)

        if imp_sha != EXPECTED_IMPLICIT_SHA[family]:
            raise ValueError(
                f"Implicit sha mismatch for {family}: got {imp_sha}, expected {EXPECTED_IMPLICIT_SHA[family]}"
            )
        if exp_sha != EXPECTED_EXPLICIT_SHA[family]:
            raise ValueError(
                f"Explicit sha mismatch for {family}: got {exp_sha}, expected {EXPECTED_EXPLICIT_SHA[family]}"
            )

        imp_norm = normalize_implicit(imp_bytes)
        exp_norm = normalize_explicit(exp_bytes)

        if imp_norm != exp_norm:
            raise ValueError(
                f"Pairing normalization failed for {family}: implicit and explicit normalized bytes are not byte-identical!"
            )

        paired_template_sha = compute_sha256_bytes(imp_norm)
        if paired_template_sha != EXPECTED_PAIRED_SHA[family]:
            raise ValueError(
                f"Paired template sha mismatch for {family}: got {paired_template_sha}, expected {EXPECTED_PAIRED_SHA[family]}"
            )

        pair_id = f"force-typed-any-{family.replace('_', '-')}"
        pairs_data.append(
            {
                "explicit_path": exp_rel,
                "explicit_sha256": exp_sha,
                "family": family,
                "implicit_path": imp_rel,
                "implicit_sha256": imp_sha,
                "pair_id": pair_id,
                "paired_template_sha256": paired_template_sha,
            }
        )

        fixture_records.append(
            {"family": family, "path": imp_rel, "sha256": imp_sha}
        )

        m = IMPLICIT_MEASURED[family]
        row = {
            "binding_identity": m["binding_identity"],
            "call_site_or_binding_site": m["call_site_or_binding_site"],
            "case_id": f"implicit_any_ingress__{family}",
            "channel": "compile",
            "expected_diagnostic_class": "ImplicitAnyIngress",
            "expected_probe_anchor": m["expected_probe_anchor"],
            "expected_terminal_classification": "rejected_at_compile_time",
            "expected_terminal_compile_result": "rejected_at_compile_time",
            "fixture_or_probe_path": imp_rel,
            "inference_path_classification": m[
                "inference_path_classification"
            ],
            "mamba_binary_sha256": MAMBA_BINARY_SHA256,
            "mamba_git_sha": MAMBA_GIT_SHA,
            "oracle_command": f"target/release/mamba check {imp_rel}",
            "oracle_executable": "target/release/mamba",
            "oracle_kind": "mamba_compile_rejection",
            "oracle_version": f"mamba {MAMBA_GIT_SHA}",
            "pair_id": pair_id,
            "paired_template_sha256": paired_template_sha,
            "platform": PLATFORM,
            "probe_id": f"implicit_any_ingress__{family}",
            "sample_role": "negative",
            "source_identity": imp_sha,
            "source_set": "implicit_any_ingress_matrix",
            "source_span": m["source_span"],
            "sut_command": f"target/release/mamba run {imp_rel}",
        }
        cases_rows.append(row)

    # Step 1: Write cases.jsonl
    cases_lines = [
        json.dumps(r, sort_keys=True) for r in cases_rows
    ]
    cases_content = "\n".join(cases_lines) + "\n"
    with open(cases_path, "w", encoding="utf-8") as f:
        f.write(cases_content)

    inventory_sha256 = compute_sha256_bytes(cases_content.encode("utf-8"))

    # Step 2: Write manifest.toml
    pairs_toml_blocks = []
    for p in pairs_data:
        block = (
            f"[[pairs]]\n"
            f'pair_id = "{p["pair_id"]}"\n'
            f'family = "{p["family"]}"\n'
            f'implicit_path = "{p["implicit_path"]}"\n'
            f'implicit_sha256 = "{p["implicit_sha256"]}"\n'
            f'explicit_path = "{p["explicit_path"]}"\n'
            f'explicit_sha256 = "{p["explicit_sha256"]}"\n'
            f'paired_template_sha256 = "{p["paired_template_sha256"]}"'
        )
        pairs_toml_blocks.append(block)

    manifest_content = (
        f"schema_version = 1\n"
        f'inventory_path = "apps/mamba/tests/governance/gates/t1_implicit_any_ingress_matrix/cases.jsonl"\n'
        f'inventory_sha256 = "{inventory_sha256}"\n'
        f"row_count = 7\n"
        f'required_channels = ["compile", "behavior", "concurrency", "performance"]\n'
        f'source_set = "implicit_any_ingress_matrix"\n'
        f"migration_inputs = [\n"
        f'    "apps/mamba/external-contracts/behavior/2011.md",\n'
        f'    "apps/mamba/tests/cpython/_regression/core/typecheck/implicit_any_ingress/",\n'
        f'    "apps/mamba/tests/cpython/_regression/core/typecheck/explicit_any_acceptance/"\n'
        f"]\n"
        f'cpython312_executable = "{CPYTHON312_EXECUTABLE}"\n'
        f'cpython312_version = "{CPYTHON312_VERSION}"\n'
        f'platform = "{PLATFORM}"\n'
        f'mamba_git_sha = "{MAMBA_GIT_SHA}"\n'
        f'mamba_binary_sha256 = "{MAMBA_BINARY_SHA256}"\n'
        f'generated_at = "{GENERATED_AT}"\n\n'
        + "\n\n".join(pairs_toml_blocks)
        + "\n"
    )

    with open(manifest_path, "w", encoding="utf-8") as f:
        f.write(manifest_content)

    manifest_sha256 = compute_sha256_bytes(manifest_content.encode("utf-8"))

    # Step 3: Write lock JSON
    lock_data = {
        "capture_timestamp": GENERATED_AT,
        "cpython312_executable": CPYTHON312_EXECUTABLE,
        "cpython312_version": CPYTHON312_VERSION,
        "evidence_path": "apps/mamba/external-contracts/evidence/mamba-t1-implicit-any-ingress-matrix-lock.json",
        "fixture_records": fixture_records,
        "generated_at": GENERATED_AT,
        "inventory_path": "apps/mamba/tests/governance/gates/t1_implicit_any_ingress_matrix/cases.jsonl",
        "inventory_sha256": inventory_sha256,
        "mamba_binary_sha256": MAMBA_BINARY_SHA256,
        "mamba_git_sha": MAMBA_GIT_SHA,
        "manifest_path": "apps/mamba/tests/governance/gates/t1_implicit_any_ingress_matrix/manifest.toml",
        "manifest_sha256": manifest_sha256,
        "migration_inputs": [
            "apps/mamba/external-contracts/behavior/2011.md",
            "apps/mamba/tests/cpython/_regression/core/typecheck/implicit_any_ingress/",
            "apps/mamba/tests/cpython/_regression/core/typecheck/explicit_any_acceptance/",
        ],
        "pairs": pairs_data,
        "platform": PLATFORM,
        "required_channels": ["compile", "behavior", "concurrency", "performance"],
        "row_count": 7,
        "schema_version": 1,
        "source_revision": MAMBA_GIT_SHA,
        "source_set": "implicit_any_ingress_matrix",
        "verifier_command": VERIFIER_COMMAND,
    }

    lock_content = json.dumps(lock_data, indent=2, sort_keys=True) + "\n"
    with open(lock_path, "w", encoding="utf-8") as f:
        f.write(lock_content)

    print(f"Generated implicit any ingress matrix successfully.")
    print(f"inventory_sha256 = {inventory_sha256}")
    print(f"manifest_sha256 = {manifest_sha256}")


if __name__ == "__main__":
    main()
