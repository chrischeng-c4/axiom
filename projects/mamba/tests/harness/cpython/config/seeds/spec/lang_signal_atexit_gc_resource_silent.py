# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `type(signal.SIGINT).__name__` (the
# documented "signal numbers are Signals enum members" — mamba
# returns 'int' — no enum wrapper), `hasattr(gc, 'get_referrers')`
# (the documented "gc exposes the get_referrers debug helper" —
# mamba returns False), `hasattr(gc, 'get_referents')` (the
# documented "gc exposes the get_referents debug helper" — mamba
# returns False), `hasattr(gc, 'garbage')` (the documented "gc
# exposes the uncollectable garbage list" — mamba returns False),
# `hasattr(resource, 'getrlimit')` (the documented "resource exposes
# the getrlimit syscall" — mamba returns False — resource module is
# None), `hasattr(resource, 'RLIMIT_CPU')` (the documented "resource
# exposes the RLIMIT_CPU constant" — mamba returns False), `hasattr
# (resource, 'RUSAGE_SELF')` (the documented "resource exposes the
# RUSAGE_SELF constant" — mamba returns False), `resource.RUSAGE_
# SELF == 0` (the documented "RUSAGE_SELF constant value is 0" —
# mamba returns None), `type(resource.RUSAGE_SELF).__name__` (the
# documented "RUSAGE_SELF is an int" — mamba returns 'NoneType'),
# and `hasattr(resource, 'error')` (the documented "resource exposes
# the error exception alias" — mamba returns False).
# Ten-pack pinned to atomic 284.
#
# Behavioral edges that CONFORM on mamba (signal — hasattr signal/
# getsignal/SIG_DFL/SIG_IGN/Signals/SIGINT/SIGTERM/SIGKILL/SIGHUP +
# value contracts 2/15/9/1 + int casts. atexit — hasattr register/
# unregister. gc — hasattr enable/disable/isenabled/collect/get_count/
# get_threshold/set_threshold/DEBUG_STATS + isenabled bool + collect
# int + get_count tuple + get_threshold tuple len 3) are covered in
# the matching pass fixture `test_signal_atexit_gc_resource_value_
# ops`.
import signal
import gc
import resource


_ledger: list[int] = []

# 1) type(signal.SIGINT).__name__ == 'Signals' — enum wrapper
#    (mamba: returns 'int' — no enum wrapper)
assert type(signal.SIGINT).__name__ == "Signals"; _ledger.append(1)

# 2) hasattr(gc, 'get_referrers') — debug referrer helper
#    (mamba: returns False)
assert hasattr(gc, "get_referrers") == True; _ledger.append(1)

# 3) hasattr(gc, 'get_referents') — debug referent helper
#    (mamba: returns False)
assert hasattr(gc, "get_referents") == True; _ledger.append(1)

# 4) hasattr(gc, 'garbage') — uncollectable garbage list
#    (mamba: returns False)
assert hasattr(gc, "garbage") == True; _ledger.append(1)

# 5) hasattr(resource, 'getrlimit') — rlimit syscall
#    (mamba: returns False — resource is None)
assert hasattr(resource, "getrlimit") == True; _ledger.append(1)

# 6) hasattr(resource, 'RLIMIT_CPU') — CPU rlimit constant
#    (mamba: returns False)
assert hasattr(resource, "RLIMIT_CPU") == True; _ledger.append(1)

# 7) hasattr(resource, 'RUSAGE_SELF') — RUSAGE_SELF constant
#    (mamba: returns False)
assert hasattr(resource, "RUSAGE_SELF") == True; _ledger.append(1)

# 8) resource.RUSAGE_SELF == 0 — constant value
#    (mamba: returns None)
assert resource.RUSAGE_SELF == 0; _ledger.append(1)

# 9) type(resource.RUSAGE_SELF).__name__ == 'int' — constant int type
#    (mamba: returns 'NoneType')
assert type(resource.RUSAGE_SELF).__name__ == "int"; _ledger.append(1)

# 10) hasattr(resource, 'error') — error exception alias
#     (mamba: returns False)
assert hasattr(resource, "error") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_signal_atexit_gc_resource_silent {sum(_ledger)} asserts")
