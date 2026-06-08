# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "ncallbacks_counts_registrations"
# subject = "atexit._ncallbacks"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit._ncallbacks: _clear()/register()/_ncallbacks() track the queue length: 0 after clear, increments per register"""
import atexit


def cleanup1():
    pass


def cleanup2():
    pass


atexit._clear()
assert atexit._ncallbacks() == 0, "queue empty after _clear()"
atexit.register(cleanup1)
assert atexit._ncallbacks() == 1, "one registration counted"
atexit.register(cleanup2)
assert atexit._ncallbacks() == 2, "two registrations counted"
atexit._clear()
assert atexit._ncallbacks() == 0, "queue empty after second _clear()"
print("ncallbacks_counts_registrations OK")
