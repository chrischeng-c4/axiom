# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "real_world"
# case = "pthread_sigmask_block_pending_unblock"
# subject = "signal.pthread_sigmask"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.pthread_sigmask: per-thread masking flow: pthread_kill delivers straight to the current thread; blocking SIGUSR1 makes a raised signal pending (sigpending reports typed Signals members) until pthread_sigmask SIG_UNBLOCK flushes it into the handler; sigwait consumes a blocked signal synchronously without invoking the handler"""
import os
import signal
import threading
import time

SIG = signal.SIGUSR1
received = []


def handler(signum, frame):
    received.append(signum)


signal.signal(SIG, handler)

# pthread_kill delivers straight to the current thread's handler.
signal.pthread_kill(threading.get_ident(), SIG)
assert received == [SIG], f"pthread_kill delivered: {received!r}"
received.clear()

# Block SIG, then raise it: the handler must NOT run yet; the signal becomes
# pending. sigpending reports typed Signals members.
old_mask = signal.pthread_sigmask(signal.SIG_BLOCK, [SIG])
assert all(isinstance(s, signal.Signals) for s in old_mask), "mask members typed"
os.kill(os.getpid(), SIG)
assert received == [], "blocked signal not yet delivered"

pending = signal.sigpending()
assert pending == {SIG}, f"sigpending = {pending!r}"
assert all(isinstance(s, signal.Signals) for s in pending), "pending typed"

# Unblocking flushes the pending signal straight into the handler.
signal.pthread_sigmask(signal.SIG_UNBLOCK, [SIG])
assert received == [SIG], f"unblock delivered pending: {received!r}"
received.clear()

# With nothing pending, sigpending is empty.
assert signal.sigpending() == set(), "no pending signals"

# sigwait: block SIG, have a helper thread raise it, and consume it
# synchronously without invoking the handler.
signal.pthread_sigmask(signal.SIG_BLOCK, [SIG])


def killer():
    time.sleep(0.2)
    os.kill(os.getpid(), SIG)


t = threading.Thread(target=killer)
t.start()
got = signal.sigwait([SIG])
t.join()
assert got == SIG, f"sigwait returned {got!r}"
assert received == [], "sigwait consumed without calling handler"
signal.pthread_sigmask(signal.SIG_UNBLOCK, [SIG])

signal.signal(SIG, signal.SIG_DFL)
print("pthread_sigmask_block_pending_unblock OK")
