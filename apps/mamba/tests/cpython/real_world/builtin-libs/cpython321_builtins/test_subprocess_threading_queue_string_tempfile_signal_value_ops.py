# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_subprocess_threading_queue_string_tempfile_signal_value_ops"
# subject = "cpython321.test_subprocess_threading_queue_string_tempfile_signal_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_subprocess_threading_queue_string_tempfile_signal_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_subprocess_threading_queue_string_tempfile_signal_value_ops: execute CPython 3.12 seed test_subprocess_threading_queue_string_tempfile_signal_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 247 pass conformance — subprocess class surface (Popen/run/call/
# check_call/check_output/DEVNULL/PIPE/STDOUT/CalledProcessError/
# TimeoutExpired/CompletedProcess/SubprocessError/getstatusoutput/
# getoutput) + PIPE/DEVNULL constant values / threading class surface
# (Thread/Lock/RLock/Event/Semaphore/BoundedSemaphore/Condition/Barrier/
# Timer/local/current_thread/main_thread/active_count/enumerate/get_ident/
# get_native_id/setprofile/settrace/stack_size/ExceptHookArgs) +
# active_count>=1 + get_ident int type / queue class surface (Queue/
# LifoQueue/PriorityQueue/SimpleQueue/Empty/Full) + Queue qsize/get
# value ops / string constants (ascii_letters/ascii_lowercase/
# ascii_uppercase/digits/hexdigits/octdigits/punctuation/whitespace/
# capwords) + value contract for each + capwords / tempfile class
# surface (NamedTemporaryFile/TemporaryFile/TemporaryDirectory/mkstemp/
# mkdtemp/gettempdir/gettempprefix/SpooledTemporaryFile) + gettempdir/
# gettempprefix str type / signal class surface (signal/SIGINT/SIGTERM/
# SIGKILL/SIGHUP/SIGUSR1/SIGUSR2/SIG_DFL/SIG_IGN/Signals/Handlers/
# alarm/getsignal/default_int_handler) that match between CPython 3.12
# and mamba.
import subprocess
import threading
import queue
import string
import tempfile
import signal


_ledger: list[int] = []

# 1) subprocess class surface
assert hasattr(subprocess, "Popen") == True; _ledger.append(1)
assert hasattr(subprocess, "run") == True; _ledger.append(1)
assert hasattr(subprocess, "call") == True; _ledger.append(1)
assert hasattr(subprocess, "check_call") == True; _ledger.append(1)
assert hasattr(subprocess, "check_output") == True; _ledger.append(1)
assert hasattr(subprocess, "DEVNULL") == True; _ledger.append(1)
assert hasattr(subprocess, "PIPE") == True; _ledger.append(1)
assert hasattr(subprocess, "STDOUT") == True; _ledger.append(1)
assert hasattr(subprocess, "CalledProcessError") == True; _ledger.append(1)
assert hasattr(subprocess, "TimeoutExpired") == True; _ledger.append(1)
assert hasattr(subprocess, "CompletedProcess") == True; _ledger.append(1)
assert hasattr(subprocess, "SubprocessError") == True; _ledger.append(1)
assert hasattr(subprocess, "getstatusoutput") == True; _ledger.append(1)
assert hasattr(subprocess, "getoutput") == True; _ledger.append(1)
assert subprocess.PIPE == -1; _ledger.append(1)
assert subprocess.DEVNULL == -3; _ledger.append(1)

# 2) threading class surface
assert hasattr(threading, "Thread") == True; _ledger.append(1)
assert hasattr(threading, "Lock") == True; _ledger.append(1)
assert hasattr(threading, "RLock") == True; _ledger.append(1)
assert hasattr(threading, "Event") == True; _ledger.append(1)
assert hasattr(threading, "Semaphore") == True; _ledger.append(1)
assert hasattr(threading, "BoundedSemaphore") == True; _ledger.append(1)
assert hasattr(threading, "Condition") == True; _ledger.append(1)
assert hasattr(threading, "Barrier") == True; _ledger.append(1)
assert hasattr(threading, "Timer") == True; _ledger.append(1)
assert hasattr(threading, "local") == True; _ledger.append(1)
assert hasattr(threading, "current_thread") == True; _ledger.append(1)
assert hasattr(threading, "main_thread") == True; _ledger.append(1)
assert hasattr(threading, "active_count") == True; _ledger.append(1)
assert hasattr(threading, "enumerate") == True; _ledger.append(1)
assert hasattr(threading, "get_ident") == True; _ledger.append(1)
assert hasattr(threading, "get_native_id") == True; _ledger.append(1)
assert hasattr(threading, "setprofile") == True; _ledger.append(1)
assert hasattr(threading, "settrace") == True; _ledger.append(1)
assert hasattr(threading, "stack_size") == True; _ledger.append(1)
assert hasattr(threading, "ExceptHookArgs") == True; _ledger.append(1)

# 3) threading callable ops
assert threading.active_count() >= 1; _ledger.append(1)
assert type(threading.get_ident()).__name__ == "int"; _ledger.append(1)

# 4) queue class surface
assert hasattr(queue, "Queue") == True; _ledger.append(1)
assert hasattr(queue, "LifoQueue") == True; _ledger.append(1)
assert hasattr(queue, "PriorityQueue") == True; _ledger.append(1)
assert hasattr(queue, "SimpleQueue") == True; _ledger.append(1)
assert hasattr(queue, "Empty") == True; _ledger.append(1)
assert hasattr(queue, "Full") == True; _ledger.append(1)

# 5) Queue value ops — qsize / get
_q: queue.Queue = queue.Queue()
_q.put(1); _q.put(2); _q.put(3)
assert _q.qsize() == 3; _ledger.append(1)
assert _q.get() == 1; _ledger.append(1)

# 6) string module hasattr surface
assert hasattr(string, "ascii_letters") == True; _ledger.append(1)
assert hasattr(string, "ascii_lowercase") == True; _ledger.append(1)
assert hasattr(string, "ascii_uppercase") == True; _ledger.append(1)
assert hasattr(string, "digits") == True; _ledger.append(1)
assert hasattr(string, "hexdigits") == True; _ledger.append(1)
assert hasattr(string, "octdigits") == True; _ledger.append(1)
assert hasattr(string, "punctuation") == True; _ledger.append(1)
assert hasattr(string, "whitespace") == True; _ledger.append(1)
assert hasattr(string, "capwords") == True; _ledger.append(1)

# 7) string constant values
assert string.ascii_letters == "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"; _ledger.append(1)
assert string.ascii_lowercase == "abcdefghijklmnopqrstuvwxyz"; _ledger.append(1)
assert string.ascii_uppercase == "ABCDEFGHIJKLMNOPQRSTUVWXYZ"; _ledger.append(1)
assert string.digits == "0123456789"; _ledger.append(1)
assert string.hexdigits == "0123456789abcdefABCDEF"; _ledger.append(1)
assert string.octdigits == "01234567"; _ledger.append(1)
assert string.punctuation == "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~"; _ledger.append(1)
assert string.capwords("hello world") == "Hello World"; _ledger.append(1)

# 8) tempfile class surface
assert hasattr(tempfile, "NamedTemporaryFile") == True; _ledger.append(1)
assert hasattr(tempfile, "TemporaryFile") == True; _ledger.append(1)
assert hasattr(tempfile, "TemporaryDirectory") == True; _ledger.append(1)
assert hasattr(tempfile, "mkstemp") == True; _ledger.append(1)
assert hasattr(tempfile, "mkdtemp") == True; _ledger.append(1)
assert hasattr(tempfile, "gettempdir") == True; _ledger.append(1)
assert hasattr(tempfile, "gettempprefix") == True; _ledger.append(1)
assert hasattr(tempfile, "SpooledTemporaryFile") == True; _ledger.append(1)
assert type(tempfile.gettempdir()).__name__ == "str"; _ledger.append(1)
assert type(tempfile.gettempprefix()).__name__ == "str"; _ledger.append(1)

# 9) signal class surface
assert hasattr(signal, "signal") == True; _ledger.append(1)
assert hasattr(signal, "SIGINT") == True; _ledger.append(1)
assert hasattr(signal, "SIGTERM") == True; _ledger.append(1)
assert hasattr(signal, "SIGKILL") == True; _ledger.append(1)
assert hasattr(signal, "SIGHUP") == True; _ledger.append(1)
assert hasattr(signal, "SIGUSR1") == True; _ledger.append(1)
assert hasattr(signal, "SIGUSR2") == True; _ledger.append(1)
assert hasattr(signal, "SIG_DFL") == True; _ledger.append(1)
assert hasattr(signal, "SIG_IGN") == True; _ledger.append(1)
assert hasattr(signal, "Signals") == True; _ledger.append(1)
assert hasattr(signal, "Handlers") == True; _ledger.append(1)
assert hasattr(signal, "alarm") == True; _ledger.append(1)
assert hasattr(signal, "getsignal") == True; _ledger.append(1)
assert hasattr(signal, "default_int_handler") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_subprocess_threading_queue_string_tempfile_signal_value_ops {sum(_ledger)} asserts")
