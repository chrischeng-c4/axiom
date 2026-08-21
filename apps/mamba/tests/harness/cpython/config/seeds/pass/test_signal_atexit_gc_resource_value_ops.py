# Atomic 284 pass conformance — signal module (hasattr signal/
# getsignal/SIGINT/SIGTERM/SIGKILL/SIGHUP/SIG_DFL/SIG_IGN/Signals +
# SIGINT==2/SIGTERM==15/SIGKILL==9/SIGHUP==1 + int casts) + atexit
# module (hasattr register/unregister) + gc module (hasattr enable/
# disable/isenabled/collect/get_count/get_threshold/set_threshold/
# DEBUG_STATS + isenabled returns bool + collect returns int +
# get_count/get_threshold return tuple + threshold tuple len 3).
# All asserts match between CPython 3.12 and mamba.
import signal
import atexit
import gc


_ledger: list[int] = []

# 1) signal — hasattr handler surface
assert hasattr(signal, "signal") == True; _ledger.append(1)
assert hasattr(signal, "getsignal") == True; _ledger.append(1)
assert hasattr(signal, "SIG_DFL") == True; _ledger.append(1)
assert hasattr(signal, "SIG_IGN") == True; _ledger.append(1)
assert hasattr(signal, "Signals") == True; _ledger.append(1)

# 2) signal — hasattr standard signal numbers (POSIX subset)
assert hasattr(signal, "SIGINT") == True; _ledger.append(1)
assert hasattr(signal, "SIGTERM") == True; _ledger.append(1)
assert hasattr(signal, "SIGKILL") == True; _ledger.append(1)
assert hasattr(signal, "SIGHUP") == True; _ledger.append(1)

# 3) signal — value contracts (int-compatible)
assert signal.SIGINT == 2; _ledger.append(1)
assert signal.SIGTERM == 15; _ledger.append(1)
assert signal.SIGKILL == 9; _ledger.append(1)
assert signal.SIGHUP == 1; _ledger.append(1)
assert int(signal.SIGINT) == 2; _ledger.append(1)
assert int(signal.SIGTERM) == 15; _ledger.append(1)

# 4) atexit — hasattr register surface
assert hasattr(atexit, "register") == True; _ledger.append(1)
assert hasattr(atexit, "unregister") == True; _ledger.append(1)

# 5) gc — hasattr enable/disable + collect surface
assert hasattr(gc, "enable") == True; _ledger.append(1)
assert hasattr(gc, "disable") == True; _ledger.append(1)
assert hasattr(gc, "isenabled") == True; _ledger.append(1)
assert hasattr(gc, "collect") == True; _ledger.append(1)
assert hasattr(gc, "get_count") == True; _ledger.append(1)
assert hasattr(gc, "get_threshold") == True; _ledger.append(1)
assert hasattr(gc, "set_threshold") == True; _ledger.append(1)
assert hasattr(gc, "DEBUG_STATS") == True; _ledger.append(1)

# 6) gc — value contracts
assert isinstance(gc.isenabled(), bool) == True; _ledger.append(1)
assert isinstance(gc.collect(), int) == True; _ledger.append(1)
assert isinstance(gc.get_count(), tuple) == True; _ledger.append(1)
assert isinstance(gc.get_threshold(), tuple) == True; _ledger.append(1)
assert len(gc.get_threshold()) == 3; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_signal_atexit_gc_resource_value_ops {sum(_ledger)} asserts")
