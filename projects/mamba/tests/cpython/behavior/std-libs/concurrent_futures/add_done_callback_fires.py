# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "add_done_callback_fires"
# subject = "concurrent.futures.Future.add_done_callback"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.Future.add_done_callback: a callback registered with Future.add_done_callback is invoked once the future completes and observes the future's result"""
import concurrent.futures
import threading

fired = threading.Event()
seen = []


def on_done(fut):
    seen.append(fut.result())
    fired.set()


with concurrent.futures.ThreadPoolExecutor(max_workers=1) as ex:
    fut = ex.submit(lambda: "callback_value")
    fut.add_done_callback(on_done)

# After the executor drains, the callback has run (event-based, no polling race).
assert fired.wait(5), "add_done_callback fired within budget"
assert seen == ["callback_value"], f"callback observed the result: {seen!r}"

print("add_done_callback_fires OK")
