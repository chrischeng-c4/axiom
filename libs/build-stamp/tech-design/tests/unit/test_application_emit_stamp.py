from __future__ import annotations

from pathlib import Path
import sys
import unittest

SRC_ROOT = Path(__file__).resolve().parents[2] / "src"
if str(SRC_ROOT) not in sys.path:
    sys.path.insert(0, str(SRC_ROOT))

from build_stamp.application.emit_stamp import StampRequest, StampService
from build_stamp.infrastructure.manual_clock import ManualClock
from build_stamp.infrastructure.set_path_probe import SetPathProbe
from build_stamp.infrastructure.static_sha_source import StaticShaSource
from build_stamp.infrastructure.static_target_source import StaticTargetSource


class TestApplicationEmitStamp(unittest.TestCase):
    def test_emission_baseline_with_hint(self) -> None:
        sha = StaticShaSource(True, b"c3ff13cd\n")
        clock = ManualClock(1700000000)
        target = StaticTargetSource("aarch64-apple-darwin")
        probe = SetPathProbe({"../../.git/HEAD"})
        service = StampService(sha, clock, target, probe)

        req = StampRequest("LUMEN", "../../.git/HEAD")
        plan = service.plan(req)
        rendered = plan.render()

        expected = (
            "cargo:rerun-if-changed=../../.git/HEAD",
            "cargo:rustc-env=LUMEN_GIT_SHA=c3ff13cd",
            "cargo:rustc-env=LUMEN_BUILT_AT=1700000000",
            "cargo:rustc-env=LUMEN_TARGET=aarch64-apple-darwin",
        )
        self.assertEqual(rendered, expected)

    def test_emission_baseline_without_hint(self) -> None:
        sha = StaticShaSource(True, b"c3ff13cd\n")
        clock = ManualClock(1700000000)
        target = StaticTargetSource("aarch64-apple-darwin")
        probe = SetPathProbe(frozenset())
        service = StampService(sha, clock, target, probe)

        req = StampRequest("LUMEN", "../../.git/HEAD")
        plan = service.plan(req)
        rendered = plan.render()

        expected = (
            "cargo:rustc-env=LUMEN_GIT_SHA=c3ff13cd",
            "cargo:rustc-env=LUMEN_BUILT_AT=1700000000",
            "cargo:rustc-env=LUMEN_TARGET=aarch64-apple-darwin",
        )
        self.assertEqual(rendered, expected)

    def test_emission_full_degradation(self) -> None:
        sha = StaticShaSource(False, b"")
        clock = ManualClock(None)
        target = StaticTargetSource(None)
        probe = SetPathProbe(frozenset())
        service = StampService(sha, clock, target, probe)

        req = StampRequest("LUMEN", "../../.git/HEAD")
        plan = service.plan(req)
        rendered = plan.render()

        expected = (
            "cargo:rustc-env=LUMEN_GIT_SHA=unknown",
            "cargo:rustc-env=LUMEN_BUILT_AT=unknown",
            "cargo:rustc-env=LUMEN_TARGET=unknown",
        )
        self.assertEqual(rendered, expected)

    def test_prefix_with_control_character(self) -> None:
        sha = StaticShaSource(True, b"c3ff13cd\n")
        clock = ManualClock(1700000000)
        target = StaticTargetSource("aarch64-apple-darwin")
        probe = SetPathProbe(frozenset())
        service = StampService(sha, clock, target, probe)

        req = StampRequest("LU\nMEN", "../../.git/HEAD")
        plan = service.plan(req)
        rendered = plan.render()

        expected = (
            "cargo:rustc-env=LUMEN_GIT_SHA=unknown",
            "cargo:rustc-env=LUMEN_BUILT_AT=unknown",
            "cargo:rustc-env=LUMEN_TARGET=unknown",
        )
        self.assertEqual(rendered, expected)

    def test_injection_floor_protection(self) -> None:
        injection_payloads = [
            "a\nb",
            "a\r\nb",
            "a\rb",
            "cargo:rustc-link-arg=-Wl,-rpath,/tmp",
            "a\ncargo:rustc-cfg=owned",
        ]
        forbidden_prefixes = (
            "cargo:rustc-link-",
            "cargo:rustc-cfg",
            "cargo:rustc-flags",
            "cargo:warning",
            "cargo:error",
        )

        for payload in injection_payloads:
            # Test in prefix
            sha = StaticShaSource(True, b"c3ff13cd\n")
            clock = ManualClock(1700000000)
            target = StaticTargetSource("aarch64-apple-darwin")
            probe = SetPathProbe({"../../.git/HEAD"})

            service = StampService(sha, clock, target, probe)

            # In prefix
            plan_prefix = service.plan(StampRequest(payload, "../../.git/HEAD"))
            lines = plan_prefix.render()
            self.assertEqual(len(lines), 4)
            for line in lines:
                self.assertFalse(
                    any(line.startswith(p) for p in forbidden_prefixes),
                    f"Line {line} started with forbidden prefix for payload {payload}",
                )

            # In sha
            sha_inj = StaticShaSource(True, payload.encode("utf-8"))
            service_sha = StampService(sha_inj, clock, target, probe)
            plan_sha = service_sha.plan(StampRequest("LUMEN", "../../.git/HEAD"))
            lines_sha = plan_sha.render()
            self.assertEqual(len(lines_sha), 4)
            for line in lines_sha:
                self.assertFalse(
                    any(line.startswith(p) for p in forbidden_prefixes),
                    f"Line {line} started with forbidden prefix for payload {payload}",
                )

            # In target
            target_inj = StaticTargetSource(payload)
            service_target = StampService(sha, clock, target_inj, probe)
            plan_target = service_target.plan(StampRequest("LUMEN", "../../.git/HEAD"))
            lines_target = plan_target.render()
            self.assertEqual(len(lines_target), 4)
            for line in lines_target:
                self.assertFalse(
                    any(line.startswith(p) for p in forbidden_prefixes),
                    f"Line {line} started with forbidden prefix for payload {payload}",
                )


if __name__ == "__main__":
    unittest.main()
