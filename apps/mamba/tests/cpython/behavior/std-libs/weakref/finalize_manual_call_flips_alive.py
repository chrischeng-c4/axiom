# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "finalize_manual_call_flips_alive"
# subject = "weakref.finalize"
# kind = "semantic"
# xfail = "mamba refcount-only: finalize() manual call does not run callback or flip alive (gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.finalize: calling a finalize manually runs the callback once and flips alive to False"""
import weakref


class _Node:
    def __init__(self, val):
        self.val = val


fired = []
n = _Node(8)
fin = weakref.finalize(n, lambda: fired.append("manual"))
fin.atexit = False  # don't run at program exit
fin()  # manual call
assert fired == ["manual"], f"manual finalize = {fired!r}"
assert not fin.alive, "finalize.alive False after manual call"

print("finalize_manual_call_flips_alive OK")
