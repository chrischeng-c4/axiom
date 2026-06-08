# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threadsignals"
# dimension = "behavior"
# case = "thread_signals__test_signals"
# subject = "cpython.test_threadsignals.ThreadSignals.test_signals"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_threadsignals.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_threadsignals.py::ThreadSignals::test_signals
"""Auto-ported test: ThreadSignals::test_signals (CPython 3.12 oracle)."""


import _thread
import signal
import sys
import threading
import time


if sys.platform.startswith("win"):
    print(f"ThreadSignals::test_signals: skipped, unsupported platform {sys.platform}")
    raise SystemExit(0)

if not all(hasattr(signal, name) for name in ("SIGUSR1", "SIGUSR2")):
    print("ThreadSignals::test_signals: skipped, SIGUSR1/SIGUSR2 unavailable")
    raise SystemExit(0)

signal_blackboard = {
    signal.SIGUSR1: {"tripped": 0, "tripped_by": 0},
    signal.SIGUSR2: {"tripped": 0, "tripped_by": 0},
}


def handle_signals(sig, frame):
    signal_blackboard[sig]["tripped"] += 1
    signal_blackboard[sig]["tripped_by"] = _thread.get_ident()


old_usr1 = signal.signal(signal.SIGUSR1, handle_signals)
old_usr2 = signal.signal(signal.SIGUSR2, handle_signals)

try:
    done = threading.Event()

    def send_signals():
        signal.raise_signal(signal.SIGUSR1)
        signal.raise_signal(signal.SIGUSR2)
        done.set()

    worker = threading.Thread(target=send_signals)
    worker.start()
    assert done.wait(5), "signal sender thread did not finish"
    worker.join(5)
    assert not worker.is_alive(), "signal sender thread did not join"

    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        if (
            signal_blackboard[signal.SIGUSR1]["tripped"] == 1
            and signal_blackboard[signal.SIGUSR2]["tripped"] == 1
        ):
            break
        time.sleep(0.01)

    main_ident = _thread.get_ident()
    assert signal_blackboard[signal.SIGUSR1]["tripped"] == 1, signal_blackboard
    assert signal_blackboard[signal.SIGUSR1]["tripped_by"] == main_ident, signal_blackboard
    assert signal_blackboard[signal.SIGUSR2]["tripped"] == 1, signal_blackboard
    assert signal_blackboard[signal.SIGUSR2]["tripped_by"] == main_ident, signal_blackboard
finally:
    signal.signal(signal.SIGUSR1, old_usr1)
    signal.signal(signal.SIGUSR2, old_usr2)

print("ThreadSignals::test_signals: ok")
