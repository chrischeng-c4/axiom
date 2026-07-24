import time

from _contract import ROOT, import_user, write_evidence


started = time.monotonic()
import_user(ROOT / "src")
elapsed_ms = int((time.monotonic() - started) * 1000)
assert elapsed_ms < 2_000
write_evidence(
    "efficiency.json",
    {"elapsed_ms": elapsed_ms, "limit_ms": 2_000, "passed": True},
)
