# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of `isinstance(threading.main_thread(),
# threading.Thread)` (the documented "main_thread() returns the principal
# Thread instance" identity contract — mamba silently returns False),
# `threading.Lock().acquire / threading.Lock().release` (the documented
# instance-method surface — mamba's Lock has no acquire/release methods,
# raises AttributeError at the call site), `threading.Event().set /
# threading.Event().is_set / threading.Event().clear` (the documented
# instance-method surface — mamba's Event has no set/is_set/clear
# methods, raises AttributeError at the call site), `hasattr(string,
# "printable")` (the documented constant — mamba does not expose it),
# `type(signal.SIGINT).__name__ == "Signals"` and `type(signal.SIGTERM
# ).__name__ == "Signals"` (the documented "signal numbers are Signals
# enum members" type contract — mamba binds them to bare ints, so
# `type(SIGINT).__name__` silently returns 'int'). Ten-pack pinned to
# atomic 247.
#
# Behavioral edges that CONFORM on mamba (subprocess 14-name surface +
# PIPE -1 + DEVNULL -3; threading 20-name surface + active_count>=1 +
# get_ident int type; queue 6-name surface + Queue qsize/get value
# ops; string 9-name surface + 7 constant values + capwords;
# tempfile 8-name surface + gettempdir/gettempprefix str type; signal
# 14-name surface) are covered in the matching pass fixture
# `test_subprocess_threading_queue_string_tempfile_signal_value_ops`.
from typing import Any
import threading as _threading_mod
import string as _string_mod
import signal as _signal_mod

threading_mod: Any = _threading_mod
string_mod: Any = _string_mod
signal_mod: Any = _signal_mod


_ledger: list[int] = []

# 1) threading.main_thread() identity
#    (mamba: silently returns False)
assert isinstance(threading_mod.main_thread(), threading_mod.Thread) == True; _ledger.append(1)

# 2) threading.Lock() instance — acquire / release methods
#    (mamba: AttributeError 'Lock' object has no attribute 'acquire')
_lk: Any = threading_mod.Lock()
assert _lk.acquire() == True; _ledger.append(1)
_lk.release()
_ledger.append(1)

# 3) threading.Event() instance — set / is_set / clear methods
#    (mamba: AttributeError 'Event' object has no attribute 'set')
_ev: Any = threading_mod.Event()
_ev.set()
_ledger.append(1)
assert _ev.is_set() == True; _ledger.append(1)
_ev.clear()
assert _ev.is_set() == False; _ledger.append(1)

# 4) string.printable — documented constant
#    (mamba: missing)
assert hasattr(string_mod, "printable") == True; _ledger.append(1)

# 5) signal.SIGINT / SIGTERM — Signals enum type contract
#    (mamba: bound to bare int, type-name silently returns 'int')
assert type(signal_mod.SIGINT).__name__ == "Signals"; _ledger.append(1)
assert type(signal_mod.SIGTERM).__name__ == "Signals"; _ledger.append(1)

# 6) signal.SIGKILL / SIGHUP — Signals enum type contract
#    (mamba: bound to bare int, type-name silently returns 'int')
assert type(signal_mod.SIGKILL).__name__ == "Signals"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_threading_string_signal_silent {sum(_ledger)} asserts")
