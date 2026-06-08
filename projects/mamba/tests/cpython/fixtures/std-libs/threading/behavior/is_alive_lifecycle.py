# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "is_alive_lifecycle"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: Thread.is_alive() is False before start, True while running, and False again after join"""
import threading

import time

_ta = threading.Thread(target=lambda: time.sleep(0.02))
assert not _ta.is_alive(), "before start: not alive"
_ta.start()
assert _ta.is_alive(), "during run: alive"
_ta.join()
assert not _ta.is_alive(), "after join: not alive"

print("is_alive_lifecycle OK")
