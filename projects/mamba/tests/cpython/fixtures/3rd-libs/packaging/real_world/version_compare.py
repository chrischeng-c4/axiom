"""Compare versions and match specifiers via `packaging.version` / `packaging.specifiers`.

End-user scenario: a downstream installer or dependency resolver
parses PEP 440 version strings, orders them, and checks them against
specifier sets (`>=1.2,<2`). This is the smallest reproducible
"packaging runs unchanged" gate — anything beyond ordering + specifier
match belongs in a larger fixture.

DoD: this script must exit 0 under both CPython and mamba.
"""

from packaging.version import Version, parse
from packaging.specifiers import SpecifierSet

# 1. Parsing + total ordering.
v1 = Version("1.2.3")
v2 = parse("1.2.10")  # `parse` returns Version for well-formed PEP 440 strings.
v3 = Version("2.0.0a1")
v4 = Version("2.0.0")

assert v1 < v2, f"1.2.3 should be less than 1.2.10: {v1} >= {v2}"
assert v2 < v3, f"1.2.10 should be less than 2.0.0a1: {v2} >= {v3}"
assert v3 < v4, f"2.0.0a1 (pre-release) should be less than 2.0.0: {v3} >= {v4}"

# 2. Specifier match — the field-tested form an installer would use.
spec = SpecifierSet(">=1.2,<2")
assert v1 in spec, f"1.2.3 should match >=1.2,<2"
assert v2 in spec, f"1.2.10 should match >=1.2,<2"
assert v4 not in spec, f"2.0.0 should NOT match >=1.2,<2"
# Pre-releases are excluded by default; explicit opt-in proves the knob works.
assert v3 not in spec, f"2.0.0a1 should NOT match >=1.2,<2 without prereleases=True"
assert v3 in SpecifierSet(">=1.2,<3", prereleases=True), (
    "2.0.0a1 should match >=1.2,<3 with prereleases=True"
)

print("ok:", str(v1), str(v2), str(v4))
