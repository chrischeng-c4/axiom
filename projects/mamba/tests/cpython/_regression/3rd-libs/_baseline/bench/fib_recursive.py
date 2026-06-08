"""Recursive Fibonacci — pure-Python baseline for the cross-runtime
bench harness (Phase 1.C, issue #1265 tier:compute).

End-user scenario: a textbook recursive workload that exercises function
call overhead and integer arithmetic — the canonical compute-tier micro
that should run faster under mamba per the #1265 target.

DoD: this script must exit 0 under both CPython and mamba with the same
printed result. Placed under `3rd-libs/_baseline/` (underscore prefix marks
it as a synthetic baseline, not a real PyPI library) so the harness has a
green ratio to report even before mamba supports third-party imports.
Once real 3rd-libs libraries (idna, urllib3, …) run end-to-end under mamba,
add their fixtures alongside this one and they will be picked up
automatically by the harness.
"""


def fib(n: int) -> int:
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)


result: int = fib(25)
print("fib:", result)
