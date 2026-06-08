# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "subclass_run_override"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: subclassing Thread and overriding run() (no target) runs the overridden body on start()"""
import threading

class _Counter(threading.Thread):
    def __init__(self):
        super().__init__()
        self.total = 0
    def run(self):
        for _ in range(10):
            self.total += 1

ct = _Counter()
ct.start()
ct.join()
assert ct.total == 10, f"subclass run() total = {ct.total!r}"

print("subclass_run_override OK")
