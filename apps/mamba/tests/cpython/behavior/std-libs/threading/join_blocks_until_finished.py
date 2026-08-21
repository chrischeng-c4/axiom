# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "join_blocks_until_finished"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: Thread.join() blocks until the worker finishes, so a worker append precedes the post-join main append in order"""
import threading

import time

_order = []

def _slow():
    time.sleep(0.01)
    _order.append("thread")

t = threading.Thread(target=_slow)
t.start()
t.join()
_order.append("main")
assert _order == ["thread", "main"], f"join order = {_order!r}"

print("join_blocks_until_finished OK")
