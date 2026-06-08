# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "behavior"
# case = "process_terminate_stops_running"
# subject = "multiprocessing.Process"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/_test_multiprocessing.py"
# status = "filled"
# ///
"""multiprocessing.Process: a daemon Process running a spin loop reports is_alive() True after start, and after terminate()+join() reports is_alive() False (spawn-guarded under __main__)"""
import multiprocessing
import time


def _spin():
    while True:
        time.sleep(0.01)


if __name__ == "__main__":
    p = multiprocessing.Process(target=_spin, daemon=True)
    p.start()
    assert p.is_alive(), "process is alive after start"
    p.terminate()
    p.join(timeout=5)
    assert not p.is_alive(), "terminated process is not alive after join"

    print("process_terminate_stops_running OK")
