# third-party — external contract

Domain map: `tech-design/stdlib/ARCHITECTURE.md` until a dedicated
third-party architecture is introduced. Verdict law: `HARNESS.md`.

Tier 6 readiness is package- and version-specific. A package is never ready
from an import-only probe, a sentinel shim, an attribute-identity assertion, or
a hand-written fake module. The evidence unit is a pinned real distribution,
its install or native build, selected upstream tests, and at least one
non-trivial user journey.

## Route contract

Every selected package must declare exactly one current route:

- pure-Python wheel or sdist;
- pure-Python fallback exposed by the upstream package;
- rebuild against the Mamba native-extension SDK;
- Mamba-native replacement with an explicit compatibility surface;
- bounded CPython/C-API emulation; or
- explicitly unsupported with an actionable diagnostic.

The route records the pinned package version, transitive native blockers,
Force Typed boundary behavior, free-threading consequences, fallback or
rollback path, and its owning GitHub issue.

## Required evidence

For each ready package row:

1. install or build from a clean Mamba environment;
2. confirm the imported distribution name and version are the real selected
   package, not a shim;
3. run the selected upstream slice;
4. run a non-trivial end-to-end journey with result assertions;
5. run applicable concurrency, cancellation, leak, CPU, and peak-RSS gates;
6. record the exact command, artifact, platform, build SHA, and owning issue.

## Gate inventory

- Tier 6 route and package issues: `#2085` through `#2099`
- Tier 6 fail-closed exit gate: `#2096`
- Native-extension SDK: `#2092`
- Mamba native kits: `apps/mamba/mambalibs/`
- Package and integration harness: `apps/mamba/tests/harness/`

Missing, skipped, xfailed, stale, wrong-version, sentinel, and import-only rows
are red. Tier 7 is not dependency-ready until the selected Tier 6 package
contract is green.
