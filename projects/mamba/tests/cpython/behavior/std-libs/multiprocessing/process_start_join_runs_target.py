# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "behavior"
# case = "process_start_join_runs_target"
# subject = "multiprocessing.Process"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/_test_multiprocessing.py"
# status = "filled"
# ///
"""multiprocessing.Process: Process.start() then join() executes the module-level target in a child; the child puts 42 on a Queue and the parent reads it back with exitcode 0 (spawn-guarded under __main__)"""
import multiprocessing


# Worker must be module-level so the spawn start method can pickle/re-import it.
def _put_value(q, val):
    q.put(val)
    q.put(None)  # sentinel


if __name__ == "__main__":
    q = multiprocessing.Queue()
    p = multiprocessing.Process(target=_put_value, args=(q, 42))
    p.start()
    p.join(timeout=10)
    assert p.exitcode == 0, f"exitcode = {p.exitcode!r}"
    got = q.get(timeout=5)
    assert got == 42, f"received = {got!r}"
    q.get(timeout=5)  # drain the sentinel

    print("process_start_join_runs_target OK")
