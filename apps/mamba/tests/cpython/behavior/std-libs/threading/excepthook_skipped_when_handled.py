# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "excepthook_skipped_when_handled"
# subject = "threading.excepthook"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.excepthook: a Thread subclass that catches its own exception inside run() never reaches threading.excepthook"""
import threading

original = threading.excepthook
seen = []
threading.excepthook = lambda args: seen.append(args.exc_type)
try:
    class Caught(threading.Thread):
        def __init__(self):
            super().__init__()
            self.exc = None
        def run(self):
            try:
                raise RuntimeError("handled")
            except RuntimeError as e:
                self.exc = e

    c = Caught()
    c.start()
    c.join()
finally:
    threading.excepthook = original

assert seen == [], "hook not called when run() handles its own exception"
assert isinstance(c.exc, RuntimeError), f"caught exc = {c.exc!r}"

print("excepthook_skipped_when_handled OK")
