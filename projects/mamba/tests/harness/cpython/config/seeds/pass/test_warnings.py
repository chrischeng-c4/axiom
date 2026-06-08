# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: warnings — warn(), simplefilter / filterwarnings / resetwarnings,
# catch_warnings context manager. All callable forms succeed without raising
# (the warning is emitted to stderr but does not interrupt the program).
# The warning *class hierarchy* (warnings.Warning, UserWarning,
# DeprecationWarning, ...) is not exposed as classes on mamba today
# (warnings.Warning resolves to None and UserWarning is not an attribute of
# the module); intentionally NOT exercised here, tracked separately.
import warnings

_ledger: list[int] = []

# warnings.warn returns None and does not raise
_r = warnings.warn("test-warn")
assert _r is None, "warnings.warn returns None"
_ledger.append(1)

# simplefilter accepts the canonical 'ignore' action
_r = warnings.simplefilter("ignore")
assert _r is None, "warnings.simplefilter('ignore') returns None"
_ledger.append(1)

# simplefilter also accepts 'default'
_r = warnings.simplefilter("default")
assert _r is None, "warnings.simplefilter('default') returns None"
_ledger.append(1)

# filterwarnings accepts the canonical 'ignore' action
_r = warnings.filterwarnings("ignore")
assert _r is None, "warnings.filterwarnings('ignore') returns None"
_ledger.append(1)

# resetwarnings clears the filter list and returns None
_r = warnings.resetwarnings()
assert _r is None, "warnings.resetwarnings() returns None"
_ledger.append(1)

# catch_warnings used as a context manager runs its body without raising
_ran = False
with warnings.catch_warnings():
    warnings.simplefilter("ignore")
    warnings.warn("inside-catch")
    _ran = True
assert _ran, "warnings.catch_warnings() body executed"
_ledger.append(1)

# A second catch_warnings block also runs cleanly
_ran2 = False
with warnings.catch_warnings():
    warnings.simplefilter("default")
    _ran2 = True
assert _ran2, "second warnings.catch_warnings() body executed"
_ledger.append(1)

# After the context exits we can still emit a warning at module level
_r = warnings.warn("post-catch")
assert _r is None, "warnings.warn after catch_warnings still returns None"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_warnings {sum(_ledger)} asserts")
