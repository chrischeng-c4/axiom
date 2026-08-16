"""Test-side path composition for Lumen tech-design and composed library tech-designs."""
from __future__ import annotations

import sys
from pathlib import Path

_REPO_ROOT = Path(__file__).resolve().parents[5]
_DESIGN_SRC = Path(__file__).resolve().parents[2] / "src"

_SHARED_DESIGN_CRATES = (
    "build-stamp",
    "cli-std",
    "metrics-prometheus",
    "openapi-codegen",
    "peer-tls",
    "raft-core",
    "raft-runtime",
    "service-auth",
    "service-backup",
    "service-http",
    "service-k8s",
    "service-observability",
    "storage-durable",
    "transport-h2c",
)

_SHARED_DESIGN_SRC = tuple(
    _REPO_ROOT / "libs" / crate / "tech-design" / "src"
    for crate in _SHARED_DESIGN_CRATES
)

# Lumen's own root goes on last so it is searched first: a shared crate must
# never be able to shadow a `lumen.*` module the tests name.
for _root in (*_SHARED_DESIGN_SRC, _DESIGN_SRC):
    if _root.is_dir() and str(_root) not in sys.path:
        sys.path.insert(0, str(_root))
