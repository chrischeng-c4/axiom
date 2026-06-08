# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: `def f(x, lst=[])` creates a fresh list per call on
# mamba; CPython shares the default list across calls (the classic
# gotcha). The fixture asserts CPython's shared-mutable contract; mamba
# silently "fixes" the gotcha and diverges from spec. See
# project_mamba_function_machinery_silent_divergences (#11).
"""Shared mutable default argument (CPython 3.12 oracle)."""


def f(x, lst=[]):  # noqa: B006 — deliberate
    lst.append(x)
    return lst


# CPython: [1]; mamba: [1].
r1 = f(1)
print("r1:", r1)
# CPython: [1, 2] (same list); mamba: [2] (fresh list).
r2 = f(2)
print("r2:", r2)
# CPython: True (r1 is r2). Mamba: False.
print("identity:", r1 is r2)
