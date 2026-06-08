# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "excepthook_receives_uncaught"
# subject = "threading.excepthook"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.excepthook: an exception escaping a worker's run() is delivered to a custom threading.excepthook (exc_type/exc_value/thread.name), not re-raised in the joiner"""
import threading

captured = []

def hook(args):
    captured.append((args.exc_type.__name__, str(args.exc_value), args.thread.name))

original = threading.excepthook
threading.excepthook = hook
try:
    def boom():
        raise ValueError("boom in thread")

    t = threading.Thread(target=boom, name="boomer")
    t.start()
    t.join()  # join itself does NOT re-raise the worker exception
finally:
    threading.excepthook = original

assert len(captured) == 1, f"hook calls = {captured!r}"
exc_name, exc_msg, thread_name = captured[0]
assert exc_name == "ValueError", f"exc type = {exc_name!r}"
assert exc_msg == "boom in thread", f"exc msg = {exc_msg!r}"
assert thread_name == "boomer", f"thread name = {thread_name!r}"

print("excepthook_receives_uncaught OK")
