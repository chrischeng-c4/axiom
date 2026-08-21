# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "errors"
# case = "daemonize_running_thread_raises"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: setting daemon=True on an already-running thread raises RuntimeError; gate the worker with an Event so the run is deterministic and joined"""
import threading

gate = threading.Event()

def hold():
    gate.wait()

running = threading.Thread(target=hold)
running.start()
_raised = False
try:
    running.daemon = True
except RuntimeError:
    _raised = True
finally:
    gate.set()
    running.join()
assert _raised, "expected RuntimeError when daemonizing a running thread"

print("daemonize_running_thread_raises OK")
