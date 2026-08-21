# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "exitstack_pop_all_transfers_callbacks"
# subject = "contextlib.ExitStack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.ExitStack: ExitStack.pop_all moves the registered callbacks to a fresh stack; closing the original is then a no-op and closing the new stack fires the callbacks"""
import contextlib

fired: list = []

es = contextlib.ExitStack()
es.callback(fired.append, "cb")
new_es = es.pop_all()

# After pop_all, the original holds nothing: closing it is a no-op.
es.close()
assert fired == [], "original stack must be empty after pop_all"

# The transferred callback fires only when the new stack closes.
new_es.close()
assert fired == ["cb"], fired

print("exitstack_pop_all_transfers_callbacks OK")
