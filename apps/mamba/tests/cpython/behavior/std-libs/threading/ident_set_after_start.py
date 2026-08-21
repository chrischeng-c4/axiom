# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "ident_set_after_start"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: Thread.ident is None before start() and becomes a non-None integer after the thread has run"""
import threading

fresh = threading.Thread(target=lambda: None)
assert fresh.ident is None, f"unstarted ident = {fresh.ident!r}"
fresh.start()
fresh.join()
assert fresh.ident is not None, f"started ident = {fresh.ident!r}"
assert isinstance(fresh.ident, int), f"started ident type = {type(fresh.ident)!r}"

print("ident_set_after_start OK")
