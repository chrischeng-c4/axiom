#!/usr/bin/env python3.12
"""Third-party package and C-extension readiness accounting for #711.

This is a replacement-readiness report, not a package installer. It keeps the
third-party ecosystem split by compatibility tier so pure-Python packages,
optional native accelerators, mandatory C extensions, and mamba-owned provider
packages cannot be accidentally counted as the same readiness class.
"""

from __future__ import annotations

import argparse
import json
import re
import tomllib
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from platform_readiness import (
    CPYTHON_DIR,
    EXIT_NOT_READY,
    Fixture,
    MAMBA_DIR,
    RESULTS_DB,
    detect_skip_reason,
    load_mamba_rows,
    repo_rel,
    runtime_state,
)


ECOSYSTEM_MANIFEST = MAMBA_DIR / "ecosystem_fixture_manifest.toml"
THIRD_PARTY_FIXTURE_DIR = CPYTHON_DIR / "_regression" / "3rd-libs"
THIRD_PARTY_GATES_DIR = MAMBA_DIR / "tests" / "governance" / "gates" / "third_party"
MAMBALIBS_FIXTURES_DIR = MAMBA_DIR / "tests" / "mambalibs" / "fixtures"
PROVIDER_RS = MAMBA_DIR / "src" / "pkgmanage" / "provider.rs"
SEED_SPEC_DIR = MAMBA_DIR / "tests" / "harness" / "cpython" / "config" / "seeds" / "spec"

REQUIRED_TIERS = (
    "pure_python",
    "optional_native_acceleration",
    "mandatory_c_extension",
    "mamba_native_replacement",
)

REQUIRED_MANAGED_ENV_GATES = {
    "pure_python_wheel_install": MAMBA_DIR
    / "tests/governance/gates/package_manager/pure_python_wheel_install/manifest.toml",
    "venv_site_packages_activation": MAMBA_DIR
    / "tests/governance/gates/package_manager/venv_site_packages_activation/manifest.toml",
    "direct_local_wheel": MAMBA_DIR
    / "tests/governance/gates/pkgmgr/direct_local_wheel/manifest.toml",
    "transitive_dependency_resolution": MAMBA_DIR
    / "tests/governance/gates/package_manager/transitive_dependency_resolution/manifest.toml",
    "offline_index": MAMBA_DIR
    / "tests/governance/gates/pkgmgr/index/manifest.toml",
}

PURE_PYTHON_PACKAGES = {
    "aiofiles",
    "alembic",
    "anyio",
    "attrs",
    "azure_core",
    "azure_identity",
    "azure_keyvault_secrets",
    "azure_storage_blob",
    "boto3",
    "botocore",
    "celery",
    "certifi",
    "charset_normalizer",
    "click",
    "fastapi",
    "flask",
    "google_api_core",
    "google_cloud_pubsub",
    "google_cloud_storage",
    "googleapis_common_protos",
    "grpclib",
    "gunicorn",
    "hypothesis",
    "idna",
    "jinja2",
    "jmespath",
    "jsonschema",
    "kombu",
    "marshmallow",
    "mock",
    "packaging",
    "pluggy",
    "pytest",
    "pytest_asyncio",
    "redis",
    "requests",
    "rich",
    "s3transfer",
    "sqlalchemy",
    "starlette",
    "typing_extensions",
    "urllib3",
    "uvicorn",
    "werkzeug",
}

OPTIONAL_NATIVE_ACCELERATION_PACKAGES = {
    "aiohttp",
    "markupsafe",
    "msgpack",
    "protobuf",
    "pyyaml",
}

MANDATORY_C_EXTENSION_PACKAGES = {
    "cryptography",
    "grpcio",
    "numpy",
    "orjson",
    "pandas",
    "psycopg",
    "pydantic",
    "pydantic_core",
    "pyopenssl",
}

C_EXTENSION_STRATEGY_CANDIDATES: dict[str, tuple[str, ...]] = {
    "cryptography": ("native_extension_bridge",),
    "grpcio": ("native_extension_bridge",),
    "numpy": ("mambalibs.array", "mambalibs.sci", "native_extension_bridge"),
    "orjson": ("native_extension_bridge",),
    "pandas": ("mambalibs.frame", "native_extension_bridge"),
    "psycopg": ("mambalibs.pg", "native_extension_bridge"),
    "pydantic": ("mambalibs.schema", "native_extension_bridge"),
    "pydantic_core": ("mambalibs.schema", "native_extension_bridge"),
    "pyopenssl": ("native_extension_bridge",),
}

OPTIONAL_NATIVE_STRATEGY_CANDIDATES: dict[str, tuple[str, ...]] = {
    "aiohttp": ("pure_python_fallback", "mambalibs.http"),
    "markupsafe": ("pure_python_fallback",),
    "msgpack": ("pure_python_fallback", "native_extension_bridge"),
    "protobuf": ("pure_python_fallback", "native_extension_bridge"),
    "pyyaml": ("pure_python_fallback",),
}

SCHEMA_GATE_PACKAGE_ALIASES = {
    "attr": "attrs",
    "pyyaml": "pyyaml",
    "sqlalchemy": "sqlalchemy",
    "attrs": "attrs",
    "httpx": "httpx",
    "pluggy": "pluggy",
    "urllib3": "urllib3",
    "fastapi": "fastapi",
    "starlette": "starlette",
    "yaml": "pyyaml",
}

IMPORT_RE = re.compile(r"^\s*(?:import|from)\s+([A-Za-z_][A-Za-z0-9_]*)", re.MULTILINE)

@dataclass
class PackageEvidence:
    name: str
    tier: str
    fixtures: list[Fixture] = field(default_factory=list)
    schema_manifests: list[str] = field(default_factory=list)
    ecosystem_entries: list[dict[str, Any]] = field(default_factory=list)
    seed_specs: list[str] = field(default_factory=list)
    provider_distributions: list[dict[str, str]] = field(default_factory=list)

    def fixture_dimensions(self) -> set[str]:
        return {fixture.dimension for fixture in self.fixtures}


def package_tier(name: str, provider_imports: set[str]) -> str:
    name = canonical_package_name(name)
    if name in provider_imports:
        return "mamba_native_replacement"
    if name in PURE_PYTHON_PACKAGES:
        return "pure_python"
    if name in OPTIONAL_NATIVE_ACCELERATION_PACKAGES:
        return "optional_native_acceleration"
    if name in MANDATORY_C_EXTENSION_PACKAGES:
        return "mandatory_c_extension"
    return "unclassified"


def canonical_package_name(name: str) -> str:
    return SCHEMA_GATE_PACKAGE_ALIASES.get(name, name)


def discover_provider_packages() -> tuple[list[dict[str, str]], list[dict[str, str]]]:
    if not PROVIDER_RS.exists():
        return [], [{"kind": "provider_catalog_missing", "path": repo_rel(PROVIDER_RS)}]
    text = PROVIDER_RS.read_text(encoding="utf-8", errors="replace")
    providers: list[dict[str, str]] = []
    parse_errors: list[dict[str, str]] = []
    for match in re.finditer(
        r'distribution:\s*"(?P<distribution>[^"]+)".*?'
        r'provides:\s*vec!\["(?P<provides>[^"]+)".*?'
        r'compatibility:\s*"(?P<compatibility>[^"]+)".*?'
        r'maturity:\s*"(?P<maturity>[^"]+)"',
        text,
        re.S,
    ):
        providers.append(match.groupdict())
    if "MambaProviderPackage" in text and not providers:
        parse_errors.append(
            {
                "kind": "provider_catalog_parse_error",
                "path": repo_rel(PROVIDER_RS),
                "reason": "provider catalog exists but no package records were parsed",
            }
        )
    return providers, parse_errors


def discover_fixtures() -> list[Fixture]:
    if not THIRD_PARTY_FIXTURE_DIR.exists():
        return []
    fixtures: list[Fixture] = []
    for path in sorted(THIRD_PARTY_FIXTURE_DIR.rglob("*.py")):
        rel_parts = path.relative_to(THIRD_PARTY_FIXTURE_DIR).parts
        if len(rel_parts) < 2:
            continue
        package = rel_parts[0]
        if package.startswith("_"):
            continue
        if rel_parts[1] == "bench":
            continue
        dimension = Path(rel_parts[1]).stem if rel_parts[1].endswith(".py") else rel_parts[1]
        case = path.stem
        text = path.read_text(encoding="utf-8", errors="replace")
        fixtures.append(
            Fixture(
                path=path,
                rel=repo_rel(path),
                scope="third_party",
                lib=package,
                dimension=dimension,
                case=case,
                xfail="#711 third-party fixture lacks current mamba runtime evidence",
                skip_reason=detect_skip_reason(text),
                parse_error="",
            )
        )
    return fixtures


def load_ecosystem_entries() -> tuple[dict[str, list[dict[str, Any]]], list[dict[str, str]]]:
    entries: dict[str, list[dict[str, Any]]] = defaultdict(list)
    if not ECOSYSTEM_MANIFEST.exists():
        return entries, [{"kind": "ecosystem_manifest_missing", "path": repo_rel(ECOSYSTEM_MANIFEST)}]
    try:
        parsed = tomllib.loads(ECOSYSTEM_MANIFEST.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError as exc:
        return entries, [{"kind": "ecosystem_manifest_parse_error", "path": repo_rel(ECOSYSTEM_MANIFEST), "error": str(exc)}]
    fixtures = parsed.get("fixtures", {})
    if not isinstance(fixtures, dict):
        return entries, [{"kind": "ecosystem_manifest_schema_error", "path": repo_rel(ECOSYSTEM_MANIFEST), "error": "missing [fixtures] table"}]
    for fixture_id, raw in fixtures.items():
        if not isinstance(raw, dict) or raw.get("category") != "3p":
            continue
        module = raw.get("module")
        if not isinstance(module, str) or not module:
            continue
        entries[canonical_package_name(module)].append(
            {
                "id": fixture_id,
                "relpath": raw.get("relpath", ""),
                "expected_outcome": raw.get("expected_outcome", ""),
                "command": raw.get("command", ""),
                "required_stdlib_modules": raw.get("required_stdlib_modules", []),
                "blocker": raw.get("blocker", ""),
            }
        )
    return entries, []


def load_schema_manifests() -> tuple[dict[str, list[str]], list[dict[str, str]]]:
    manifests: dict[str, list[str]] = defaultdict(list)
    errors: list[dict[str, str]] = []
    if not THIRD_PARTY_GATES_DIR.exists():
        return manifests, [{"kind": "third_party_gate_dir_missing", "path": repo_rel(THIRD_PARTY_GATES_DIR)}]
    for path in sorted(THIRD_PARTY_GATES_DIR.glob("*/manifest.toml")):
        rel = repo_rel(path)
        try:
            parsed = tomllib.loads(path.read_text(encoding="utf-8"))
        except tomllib.TOMLDecodeError as exc:
            errors.append({"kind": "third_party_manifest_parse_error", "path": rel, "error": str(exc)})
            continue
        modules = (
            parsed.get("surface", {}).get("covered_modules", [])
            if isinstance(parsed.get("surface"), dict)
            else []
        )
        if isinstance(modules, list):
            for module in modules:
                if isinstance(module, str):
                    manifests[canonical_package_name(module)].append(rel)
        family = parsed.get("family")
        if isinstance(family, str):
            for key, package in SCHEMA_GATE_PACKAGE_ALIASES.items():
                canonical = canonical_package_name(package)
                if key in family and rel not in manifests[canonical]:
                    manifests[canonical].append(rel)
    return manifests, errors


def load_seed_specs() -> dict[str, list[str]]:
    specs: dict[str, list[str]] = defaultdict(list)
    if not SEED_SPEC_DIR.exists():
        return specs
    for path in sorted(SEED_SPEC_DIR.glob("test_*.py")):
        text = path.read_text(encoding="utf-8", errors="replace")
        imported = {match.group(1) for match in IMPORT_RE.finditer(text)}
        for package in imported:
            specs[canonical_package_name(package)].append(repo_rel(path))
    return specs


def managed_env_gates() -> tuple[dict[str, str], list[dict[str, str]]]:
    found: dict[str, str] = {}
    missing: list[dict[str, str]] = []
    for gate, path in REQUIRED_MANAGED_ENV_GATES.items():
        if path.exists():
            found[gate] = repo_rel(path)
        else:
            missing.append({"kind": "missing_managed_env_gate", "gate": gate, "path": repo_rel(path)})
    return found, missing


def mambalibs_native_evidence() -> dict[str, Any]:
    fixtures = []
    if MAMBALIBS_FIXTURES_DIR.exists():
        fixtures = [
            path
            for path in sorted(MAMBALIBS_FIXTURES_DIR.iterdir())
            if path.is_dir()
        ]
    import_gates = [path for path in fixtures if path.name.endswith("_import_gate")]
    return {
        "owner_issue": "#714",
        "fixture_count": len(fixtures),
        "import_gate_count": len(import_gates),
        "fixtures_dir": repo_rel(MAMBALIBS_FIXTURES_DIR),
        "sample_import_gates": [path.name for path in import_gates[:8]],
        "native_kit_readiness_report": (
            "python3.12 projects/mamba/tests/harness/cpython/tools/"
            "mambalibs_readiness.py --json"
        ),
        "counts_as_pure_python_package": False,
        "counts_as_cpython_extension_abi": False,
        "reason": (
            "mambalibs native bindings are mamba-only replacement evidence; "
            "full native-kit replacement readiness is tracked separately by #714, "
            "and they do not prove CPython C-extension ABI compatibility"
        ),
    }


def collect_packages() -> tuple[dict[str, PackageEvidence], list[dict[str, str]]]:
    provider_rows, provider_errors = discover_provider_packages()
    provider_imports = {row["provides"] for row in provider_rows}
    packages: dict[str, PackageEvidence] = {}

    def ensure(name: str) -> PackageEvidence:
        name = canonical_package_name(name)
        if name not in packages:
            packages[name] = PackageEvidence(name=name, tier=package_tier(name, provider_imports))
        return packages[name]

    for fixture in discover_fixtures():
        ensure(fixture.lib).fixtures.append(fixture)
    ecosystem_entries, ecosystem_errors = load_ecosystem_entries()
    for package, entries in ecosystem_entries.items():
        ensure(package).ecosystem_entries.extend(entries)
    schema_manifests, schema_errors = load_schema_manifests()
    for package, paths in schema_manifests.items():
        ensure(package).schema_manifests.extend(sorted(set(paths)))
    seed_specs = load_seed_specs()
    for package, paths in seed_specs.items():
        if package in packages or package in provider_imports or package in PURE_PYTHON_PACKAGES:
            ensure(package).seed_specs.extend(paths)
    for row in provider_rows:
        ensure(row["provides"]).provider_distributions.append(row)
    return packages, [*provider_errors, *ecosystem_errors, *schema_errors]


def package_has_non_stub_fixture(package: PackageEvidence, dimensions: set[str]) -> bool:
    for fixture in package.fixtures:
        if fixture.dimension in dimensions and fixture.case != "_stub":
            return True
    return False


def package_status(
    package: PackageEvidence, rows: dict[str, dict[str, Any]], managed_gates_missing: bool
) -> tuple[str, list[dict[str, Any]], Counter[str]]:
    runtime_counts: Counter[str] = Counter()
    surface_ok = False
    behavior_ok = False
    gaps: list[dict[str, Any]] = []

    for fixture in package.fixtures:
        state = runtime_state(rows.get(fixture.rel))
        runtime_counts[state] += 1
        if fixture.dimension == "surface" and state == "runtime_ok":
            surface_ok = True
        if fixture.dimension in {"behavior", "real_world"} and fixture.case != "_stub" and state == "runtime_ok":
            behavior_ok = True
        if fixture.case == "_stub":
            gaps.append(
                {
                    "kind": "stub_real_world_fixture",
                    "package": package.name,
                    "path": fixture.rel,
                    "tier": package.tier,
                    "owner_refs": ["#711"],
                    "owned": True,
                    "reason": "real-world third-party fixture is still a stub and cannot prove package readiness",
                }
            )
        if state != "runtime_ok":
            gaps.append(
                {
                    "kind": state,
                    "package": package.name,
                    "path": fixture.rel,
                    "dimension": fixture.dimension,
                    "tier": package.tier,
                    "owner_refs": [],
                    "owned": False,
                    "reason": "no current mamba results-store row for this third-party fixture"
                    if state == "unmeasured"
                    else state,
                }
            )

    has_import_smoke = package_has_non_stub_fixture(package, {"surface"}) or bool(package.seed_specs or package.schema_manifests)
    has_behavior_smoke = package_has_non_stub_fixture(package, {"behavior", "real_world"}) or bool(package.ecosystem_entries)
    has_install_run_manifest = any(entry.get("expected_outcome") == "pass" for entry in package.ecosystem_entries)
    if package.tier in {"pure_python", "mamba_native_replacement"}:
        if managed_gates_missing:
            gaps.append(
                {
                    "kind": "managed_environment_gate_missing",
                    "package": package.name,
                    "tier": package.tier,
                    "owner_refs": ["#711"],
                    "owned": True,
                    "reason": "mamba-managed environment gates are incomplete",
                }
            )
        if not has_import_smoke:
            gaps.append(
                {
                    "kind": "missing_import_smoke",
                    "package": package.name,
                    "tier": package.tier,
                    "owner_refs": ["#711"],
                    "owned": True,
                    "reason": "package has no import smoke fixture or seed",
                }
            )
        if not has_behavior_smoke:
            gaps.append(
                {
                    "kind": "missing_behavior_smoke",
                    "package": package.name,
                    "tier": package.tier,
                    "owner_refs": ["#711"],
                    "owned": True,
                    "reason": "package has no behavioral or real-world smoke fixture",
                }
            )
        if not has_install_run_manifest:
            gaps.append(
                {
                    "kind": "missing_mamba_managed_install_run_manifest",
                    "package": package.name,
                    "tier": package.tier,
                    "owner_refs": ["#711"],
                    "owned": True,
                    "reason": "package is not represented in ecosystem_fixture_manifest.toml as a mamba run entry",
                }
            )
        if surface_ok and behavior_ok and has_install_run_manifest and not managed_gates_missing:
            return "ready", gaps, runtime_counts
        return "not_ready", gaps, runtime_counts

    if package.tier == "optional_native_acceleration":
        gaps.append(
            {
                "kind": "optional_native_acceleration_unproven",
                "package": package.name,
                "tier": package.tier,
                "strategy_candidates": list(
                    OPTIONAL_NATIVE_STRATEGY_CANDIDATES.get(
                        package.name, ("pure_python_fallback", "native_extension_bridge")
                    )
                ),
                "owner_refs": ["#711"],
                "owned": True,
                "reason": "package needs explicit proof that the pure-Python fallback is selected and native acceleration absence is safe",
            }
        )
        return "blocked_native_strategy", gaps, runtime_counts

    if package.tier == "mandatory_c_extension":
        gaps.append(
            {
                "kind": "mandatory_c_extension_unsupported",
                "package": package.name,
                "tier": package.tier,
                "strategy_candidates": list(
                    C_EXTENSION_STRATEGY_CANDIDATES.get(
                        package.name, ("native_extension_bridge",)
                    )
                ),
                "owner_refs": ["#711"],
                "owned": True,
                "reason": "package requires CPython C/Rust extension ABI support or a mamba-native replacement/bridge",
            }
        )
        return "blocked_c_extension_strategy", gaps, runtime_counts

    gaps.append(
        {
            "kind": "unclassified_third_party_package",
            "package": package.name,
            "tier": package.tier,
            "owner_refs": ["#711"],
            "owned": True,
            "reason": "package is discovered in the third-party surface but has no explicit #711 compatibility tier",
        }
    )
    return "unclassified", gaps, runtime_counts


def build_report(show: int, db: Path) -> dict[str, Any]:
    packages, catalog_errors = collect_packages()
    all_fixtures = [fixture for package in packages.values() for fixture in package.fixtures]
    rows = load_mamba_rows(all_fixtures, db)
    managed_gates, missing_managed_gates = managed_env_gates()
    mambalibs_evidence = mambalibs_native_evidence()

    by_tier: dict[str, Counter[str]] = defaultdict(Counter)
    by_package: dict[str, dict[str, Any]] = {}
    runtime_counts: Counter[str] = Counter()
    gaps: list[dict[str, Any]] = []

    for package in sorted(packages.values(), key=lambda item: item.name):
        status, package_gaps, package_runtime = package_status(
            package, rows, bool(missing_managed_gates)
        )
        runtime_counts.update(package_runtime)
        by_tier[package.tier]["packages"] += 1
        by_tier[package.tier][status] += 1
        by_tier[package.tier]["fixtures"] += len(package.fixtures)
        by_package[package.name] = {
            "tier": package.tier,
            "status": status,
            "fixture_count": len(package.fixtures),
            "fixture_dimensions": sorted(package.fixture_dimensions()),
            "schema_manifest_count": len(package.schema_manifests),
            "ecosystem_entry_count": len(package.ecosystem_entries),
            "seed_spec_count": len(package.seed_specs),
            "provider_distributions": package.provider_distributions,
            "runtime": dict(sorted(package_runtime.items())),
        }
        gaps.extend(package_gaps)

    tier_names = set(by_tier)
    missing_tiers = [
        {"kind": "missing_third_party_tier", "tier": tier}
        for tier in REQUIRED_TIERS
        if tier not in tier_names
    ]
    unowned_gap_count = sum(1 for gap in gaps if not gap.get("owned"))
    ready_packages = sum(1 for item in by_package.values() if item["status"] == "ready")
    blocked_c_extension = sum(
        1 for item in by_package.values() if item["status"] == "blocked_c_extension_strategy"
    )
    blocked_optional_native = sum(
        1 for item in by_package.values() if item["status"] == "blocked_native_strategy"
    )
    unclassified_packages = sum(1 for item in by_package.values() if item["tier"] == "unclassified")

    ready = (
        not catalog_errors
        and not missing_managed_gates
        and not missing_tiers
        and unowned_gap_count == 0
        and ready_packages == len(packages)
        and blocked_c_extension == 0
        and blocked_optional_native == 0
        and unclassified_packages == 0
    )

    blockers = [
        *catalog_errors,
        *missing_managed_gates,
        *missing_tiers,
        *[gap for gap in gaps if not gap.get("owned")][:show],
        *[gap for gap in gaps if gap.get("owned")][:show],
    ]

    return {
        "schema_version": 1,
        "owner_issue": "#711",
        "ready": ready,
        "status": "green" if ready else "red",
        "required_tiers": REQUIRED_TIERS,
        "managed_environment_gates": managed_gates,
        "mambalibs_native_binding_evidence": mambalibs_evidence,
        "results_db": repo_rel(db) if db.is_absolute() else str(db),
        "counts": {
            "packages": len(packages),
            "fixtures": len(all_fixtures),
            "tiers": len(tier_names),
            "missing_tiers": len(missing_tiers),
            "managed_environment_gates": len(managed_gates),
            "missing_managed_environment_gates": len(missing_managed_gates),
            "catalog_errors": len(catalog_errors),
            "ready_packages": ready_packages,
            "not_ready_packages": sum(1 for item in by_package.values() if item["status"] == "not_ready"),
            "blocked_c_extension_packages": blocked_c_extension,
            "blocked_optional_native_packages": blocked_optional_native,
            "unclassified_packages": unclassified_packages,
            "mamba_native_replacement_packages": by_tier["mamba_native_replacement"]["packages"],
            "pure_python_packages": by_tier["pure_python"]["packages"],
            "optional_native_acceleration_packages": by_tier["optional_native_acceleration"]["packages"],
            "mandatory_c_extension_packages": by_tier["mandatory_c_extension"]["packages"],
            "schema_manifest_packages": sum(1 for item in by_package.values() if item["schema_manifest_count"]),
            "ecosystem_required_3p_packages": sum(1 for item in by_package.values() if item["ecosystem_entry_count"]),
            "provider_packages": sum(1 for item in by_package.values() if item["provider_distributions"]),
            "mambalibs_native_fixture_count": mambalibs_evidence["fixture_count"],
            "mambalibs_native_import_gate_count": mambalibs_evidence["import_gate_count"],
            "runtime_ok": runtime_counts["runtime_ok"],
            "runtime_fail": runtime_counts["runtime_fail"],
            "runtime_timeout": runtime_counts["runtime_timeout"],
            "runtime_crash": runtime_counts["runtime_crash"],
            "unmeasured": runtime_counts["unmeasured"],
            "stub_real_world_fixtures": sum(1 for gap in gaps if gap["kind"] == "stub_real_world_fixture"),
            "missing_import_smoke_packages": sum(1 for gap in gaps if gap["kind"] == "missing_import_smoke"),
            "missing_behavior_smoke_packages": sum(1 for gap in gaps if gap["kind"] == "missing_behavior_smoke"),
            "missing_install_run_manifest_packages": sum(1 for gap in gaps if gap["kind"] == "missing_mamba_managed_install_run_manifest"),
            "unowned_gap_count": unowned_gap_count,
        },
        "by_tier": {key: dict(sorted(value.items())) for key, value in sorted(by_tier.items())},
        "by_package": by_package,
        "blocker_count": len(catalog_errors)
        + len(missing_managed_gates)
        + len(missing_tiers)
        + len(gaps),
        "blockers": blockers,
        "evidence_commands": [
            "python3.12 projects/mamba/tests/harness/cpython/tools/third_party_readiness.py --json",
            "cargo test -p mamba --test schema_gates third_party_readiness_gate_711 -- --nocapture",
            "cargo test -p mamba --test schema_gates ecosystem_fixture_manifest_smoke -- --nocapture",
            "target/debug/mamba add --provider mamba mamba-httpx-compat",
            "cargo test -p mamba --test mambalibs_integration -- --nocapture",
            "target/debug/mamba run tests/cpython/_regression/3rd-libs/requests/real_world/local_adapter_roundtrip.py",
        ],
    }


def print_human(report: dict[str, Any]) -> None:
    print(f"third-party readiness: {report['status']}")
    counts = report["counts"]
    print(
        "  packages={packages} ready={ready_packages} not_ready={not_ready_packages} "
        "mandatory_c_extension={blocked_c_extension_packages} optional_native={blocked_optional_native_packages} "
        "unmeasured={unmeasured} stubs={stub_real_world_fixtures}".format(**counts)
    )
    for tier, tier_counts in report["by_tier"].items():
        print(f"- {tier}: {tier_counts}")
    for blocker in report["blockers"][:5]:
        label = blocker.get("package") or blocker.get("path") or blocker.get("tier") or blocker.get("kind")
        print(f"  blocker: {blocker['kind']} {label}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--show", type=int, default=10)
    parser.add_argument("--db", type=Path, default=RESULTS_DB)
    args = parser.parse_args(argv)

    report = build_report(args.show, args.db)
    if args.json:
        print(json.dumps(report, sort_keys=True))
    else:
        print_human(report)
    return 0 if report["ready"] else EXIT_NOT_READY


if __name__ == "__main__":
    raise SystemExit(main())
