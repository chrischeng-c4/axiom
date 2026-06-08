# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "context_manager_completes_futures"
# subject = "concurrent.futures.ThreadPoolExecutor.__exit__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.ThreadPoolExecutor.__exit__: using the executor as a context manager blocks on __exit__ until all in-flight futures are done(), so every future reports done() True after the with-block"""
import concurrent.futures

with concurrent.futures.ThreadPoolExecutor(max_workers=2) as ex:
    futs = [ex.submit(lambda x=i: x, i) for i in range(4)]

# __exit__ shut the pool down with wait=True, so every future is settled.
for f in futs:
    assert f.done() is True, "future done after executor context exit"
assert sorted(f.result() for f in futs) == [0, 1, 2, 3], "all futures carry their results"

print("context_manager_completes_futures OK")
