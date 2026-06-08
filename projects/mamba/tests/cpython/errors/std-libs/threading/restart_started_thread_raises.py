# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "errors"
# case = "restart_started_thread_raises"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: starting an already-started-and-joined thread a second time raises RuntimeError ('threads can only be started once')"""
import threading

def noop():
    pass

t = threading.Thread(target=noop)
t.start()
t.join()
_raised = False
try:
    t.start()
except RuntimeError as e:
    _raised = True
    assert "once" in str(e), f"message = {str(e)!r}"
assert _raised, "expected RuntimeError on restart"

print("restart_started_thread_raises OK")
