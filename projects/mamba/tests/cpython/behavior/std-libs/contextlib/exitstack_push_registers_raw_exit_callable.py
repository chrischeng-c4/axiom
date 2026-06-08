# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "exitstack_push_registers_raw_exit_callable"
# subject = "contextlib.ExitStack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.ExitStack: ExitStack.push registers a raw __exit__-style callable (bypassing the __enter__ check) and a push()ed callback returning True suppresses a body exception"""
import contextlib

seen: list = []


def exit_cb(*exc_details):
    seen.append(exc_details[0])  # the exc_type, or None on clean exit
    return False


# push() takes a raw __exit__-style callable; on a clean exit it is called
# with (None, None, None).
with contextlib.ExitStack() as stack:
    stack.push(exit_cb)
assert seen == [None], seen


# A push()ed callback returning True suppresses a body exception.
def suppress_all(*exc_details):
    return True


with contextlib.ExitStack() as stack:
    stack.push(suppress_all)
    1 / 0  # suppressed by the callback

print("exitstack_push_registers_raw_exit_callable OK")
