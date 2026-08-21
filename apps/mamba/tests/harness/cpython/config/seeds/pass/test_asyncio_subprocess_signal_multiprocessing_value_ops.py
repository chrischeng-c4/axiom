# Operational AssertionPass seed for the value contract of the
# `asyncio` / `concurrent.futures` / `multiprocessing` /
# `subprocess` / `signal` five-pack pinned to atomic 192:
# `asyncio` (the documented partial module-level helper hasattr
# surface — `run` / `sleep` / `gather` / `wait` / `create_task`
# / `ensure_future` + the documented asyncio.run round-trip
# value contract), `concurrent.futures` (the documented empty
# module-level helper hasattr surface — none of the documented
# class identifiers are present on mamba; pass omits the
# divergent surface and covers it in the spec fixture instead),
# `multiprocessing` (the documented partial module-level helper
# hasattr surface — `Process` / `Queue` / `cpu_count` /
# `current_process` + the documented multiprocessing.cpu_count
# return-type and positive-integer value contract), `subprocess`
# (the documented full module-level helper hasattr surface —
# `run` / `Popen` / `PIPE` / `STDOUT` / `DEVNULL` / `call` /
# `check_call` / `check_output` / `CompletedProcess` /
# `CalledProcessError` / `TimeoutExpired`), and `signal` (the
# documented full module-level helper hasattr surface —
# `SIGINT` / `SIGTERM` / `SIGKILL` / `SIGUSR1` / `SIGUSR2` /
# `SIGHUP` / `SIGALRM` / `SIGCHLD` / `signal` / `getsignal` /
# `Signals` / `Handlers` / `SIG_DFL` / `SIG_IGN` + the
# documented signal.SIGINT == 2 / signal.SIGTERM == 15
# integer-value contract).
#
# The matching subset between mamba and CPython is the partial
# `asyncio` module hasattr surface (run / sleep / gather /
# wait / create_task / ensure_future — `get_event_loop` /
# `new_event_loop` / `set_event_loop` / `Future` / `Task` /
# `Lock` / `Event` / `Queue` / `Semaphore` / `iscoroutine` /
# `iscoroutinefunction` / `TimeoutError` / `CancelledError`
# DIVERGE) + the asyncio.run round-trip value layer, the
# partial `multiprocessing` module hasattr surface (Process /
# Queue / cpu_count / current_process — `Pool` / `Pipe` /
# `Lock` / `Manager` / `Value` / `Array` DIVERGE) + the
# cpu_count value layer, the full `subprocess` module hasattr
# surface, and the full `signal` module hasattr surface + the
# SIGINT / SIGTERM integer-value layer.
#
# Surface in this fixture:
#   • asyncio — partial module hasattr surface (run / sleep
#     / gather / wait / create_task / ensure_future);
#   • asyncio.run — round-trip value contract (calls a
#     coroutine returning 42, expects 42);
#   • multiprocessing — partial module hasattr surface
#     (Process / Queue / cpu_count / current_process);
#   • multiprocessing.cpu_count — return-type + positive-
#     integer value contract;
#   • subprocess — full module hasattr surface (run /
#     Popen / PIPE / STDOUT / DEVNULL / call / check_call
#     / check_output / CompletedProcess /
#     CalledProcessError / TimeoutExpired);
#   • signal — full module hasattr surface (SIGINT /
#     SIGTERM / SIGKILL / SIGUSR1 / SIGUSR2 / SIGHUP /
#     SIGALRM / SIGCHLD / signal / getsignal / Signals /
#     Handlers / SIG_DFL / SIG_IGN);
#   • signal.SIGINT / SIGTERM — integer-value contract.
#
# Behavioral edges that DIVERGE on mamba
# (hasattr(asyncio, "get_event_loop") / "new_event_loop" /
# "set_event_loop" / "Future" / "Task" / "Lock" / "Event" /
# "Queue" / "Semaphore" / "iscoroutine" /
# "iscoroutinefunction" / "TimeoutError" / "CancelledError"
# all False, hasattr(concurrent.futures, "Future") /
# "Executor" / "ThreadPoolExecutor" / "ProcessPoolExecutor"
# / "as_completed" / "wait" / "TimeoutError" /
# "CancelledError" all False, concurrent.futures
# .ThreadPoolExecutor raises AttributeError on mamba,
# hasattr(multiprocessing, "Pool") / "Pipe" / "Lock" /
# "Manager" / "Value" / "Array" all False) are covered in
# the matching spec fixture
# `lang_asyncio_concurrent_multiprocessing_silent`.
import asyncio
import multiprocessing
import subprocess
import signal


_ledger: list[int] = []

# 1) asyncio — partial module hasattr surface
#    (get_event_loop / new_event_loop / set_event_loop /
#    Future / Task / Lock / Event / Queue / Semaphore /
#    iscoroutine / iscoroutinefunction / TimeoutError /
#    CancelledError DIVERGE — moved to spec fixture)
assert hasattr(asyncio, "run") == True; _ledger.append(1)
assert hasattr(asyncio, "sleep") == True; _ledger.append(1)
assert hasattr(asyncio, "gather") == True; _ledger.append(1)
assert hasattr(asyncio, "wait") == True; _ledger.append(1)
assert hasattr(asyncio, "create_task") == True; _ledger.append(1)
assert hasattr(asyncio, "ensure_future") == True; _ledger.append(1)

# 2) asyncio.run — round-trip value contract
async def _coro():
    return 42

assert asyncio.run(_coro()) == 42; _ledger.append(1)

# 3) multiprocessing — partial module hasattr surface
#    (Pool / Pipe / Lock / Manager / Value / Array DIVERGE —
#    moved to spec fixture)
assert hasattr(multiprocessing, "Process") == True; _ledger.append(1)
assert hasattr(multiprocessing, "Queue") == True; _ledger.append(1)
assert hasattr(multiprocessing, "cpu_count") == True; _ledger.append(1)
assert hasattr(multiprocessing, "current_process") == True; _ledger.append(1)

# 4) multiprocessing.cpu_count — value contract
_cnt = multiprocessing.cpu_count()
assert type(_cnt).__name__ == "int"; _ledger.append(1)
assert _cnt > 0; _ledger.append(1)

# 5) subprocess — full module hasattr surface
assert hasattr(subprocess, "run") == True; _ledger.append(1)
assert hasattr(subprocess, "Popen") == True; _ledger.append(1)
assert hasattr(subprocess, "PIPE") == True; _ledger.append(1)
assert hasattr(subprocess, "STDOUT") == True; _ledger.append(1)
assert hasattr(subprocess, "DEVNULL") == True; _ledger.append(1)
assert hasattr(subprocess, "call") == True; _ledger.append(1)
assert hasattr(subprocess, "check_call") == True; _ledger.append(1)
assert hasattr(subprocess, "check_output") == True; _ledger.append(1)
assert hasattr(subprocess, "CompletedProcess") == True; _ledger.append(1)
assert hasattr(subprocess, "CalledProcessError") == True; _ledger.append(1)
assert hasattr(subprocess, "TimeoutExpired") == True; _ledger.append(1)

# 6) signal — full module hasattr surface
assert hasattr(signal, "SIGINT") == True; _ledger.append(1)
assert hasattr(signal, "SIGTERM") == True; _ledger.append(1)
assert hasattr(signal, "SIGKILL") == True; _ledger.append(1)
assert hasattr(signal, "SIGUSR1") == True; _ledger.append(1)
assert hasattr(signal, "SIGUSR2") == True; _ledger.append(1)
assert hasattr(signal, "SIGHUP") == True; _ledger.append(1)
assert hasattr(signal, "SIGALRM") == True; _ledger.append(1)
assert hasattr(signal, "SIGCHLD") == True; _ledger.append(1)
assert hasattr(signal, "signal") == True; _ledger.append(1)
assert hasattr(signal, "getsignal") == True; _ledger.append(1)
assert hasattr(signal, "Signals") == True; _ledger.append(1)
assert hasattr(signal, "Handlers") == True; _ledger.append(1)
assert hasattr(signal, "SIG_DFL") == True; _ledger.append(1)
assert hasattr(signal, "SIG_IGN") == True; _ledger.append(1)

# 7) signal — integer-value contract
assert int(signal.SIGINT) == 2; _ledger.append(1)
assert int(signal.SIGTERM) == 15; _ledger.append(1)

# NB: hasattr(asyncio, "get_event_loop") / "new_event_loop"
# / "set_event_loop" / "Future" / "Task" / "Lock" / "Event"
# / "Queue" / "Semaphore" / "iscoroutine" /
# "iscoroutinefunction" / "TimeoutError" / "CancelledError"
# all False on mamba, hasattr(concurrent.futures, "Future")
# / "Executor" / "ThreadPoolExecutor" /
# "ProcessPoolExecutor" / "as_completed" / "wait" /
# "TimeoutError" / "CancelledError" all False on mamba,
# concurrent.futures.ThreadPoolExecutor raises AttributeError
# on mamba (the submodule attribute access fails outright),
# hasattr(multiprocessing, "Pool") / "Pipe" / "Lock" /
# "Manager" / "Value" / "Array" all False on mamba — all
# DIVERGE on mamba — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_asyncio_subprocess_signal_multiprocessing_value_ops {sum(_ledger)} asserts")
