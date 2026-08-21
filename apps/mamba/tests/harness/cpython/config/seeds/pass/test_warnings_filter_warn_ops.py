# Operational AssertionPass seed for `warnings` filter+warn surface.
# Surface: `warnings.warn(msg)` and `warnings.warn(msg, category)`
# both return None — calling them is a no-op for the surrounding
# program state. `warnings.filterwarnings(action)`,
# `warnings.simplefilter(action)`, and `warnings.resetwarnings()`
# return None. `catch_warnings()` is a context manager that scopes
# filter changes locally — code inside the `with` block runs to
# completion and produces the expected non-warn side-effects. The
# subclass relations of the canonical warning categories are stable
# (`DeprecationWarning`, `UserWarning`, `FutureWarning`,
# `RuntimeWarning`, `SyntaxWarning` all derive from `Warning`).
import warnings
_ledger: list[int] = []

# catch_warnings is a usable context manager
with warnings.catch_warnings():
    warnings.simplefilter("ignore")
    warnings.warn("test")
    a = 1
assert a == 1; _ledger.append(1)

# warn returns None
r = warnings.warn("a")
assert r is None; _ledger.append(1)

# warn with category returns None
r2 = warnings.warn("dep", DeprecationWarning)
assert r2 is None; _ledger.append(1)

# filterwarnings returns None
fr = warnings.filterwarnings("ignore")
assert fr is None; _ledger.append(1)

# resetwarnings returns None
rr = warnings.resetwarnings()
assert rr is None; _ledger.append(1)

# simplefilter returns None
sr = warnings.simplefilter("default")
assert sr is None; _ledger.append(1)

# Subclass relations of builtin warning classes
assert issubclass(DeprecationWarning, Warning); _ledger.append(1)
assert issubclass(UserWarning, Warning); _ledger.append(1)
assert issubclass(FutureWarning, Warning); _ledger.append(1)
assert issubclass(RuntimeWarning, Warning); _ledger.append(1)
assert issubclass(SyntaxWarning, Warning); _ledger.append(1)

# Nested catch_warnings reads filter changes locally and runs body
with warnings.catch_warnings():
    warnings.simplefilter("always")
    x = "inside"
assert x == "inside"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_warnings_filter_warn_ops {sum(_ledger)} asserts")
