# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "nested_submit_resolves"
# subject = "concurrent.futures.ThreadPoolExecutor.submit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.ThreadPoolExecutor.submit: a task may submit further tasks to the same pool and await their results; outer tasks i for range(3) returning inner i*10 give {0,10,20}"""
import concurrent.futures


def outer_task(n, executor):
    inner = executor.submit(lambda x=n: x * 10)
    return inner.result(timeout=5)


with concurrent.futures.ThreadPoolExecutor(max_workers=4) as ex:
    outer = [ex.submit(outer_task, i, ex) for i in range(3)]
    results = [f.result(timeout=5) for f in outer]

assert sorted(results) == [0, 10, 20], f"nested submit results = {sorted(results)!r}"

print("nested_submit_resolves OK")
