# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "as_completed_yields_all_futures"
# subject = "concurrent.futures.as_completed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.as_completed: as_completed yields every submitted future exactly once as it finishes; collecting results over range(5) squares gives the full set {0,1,4,9,16}"""
import concurrent.futures


def square(n):
    return n * n


with concurrent.futures.ThreadPoolExecutor(max_workers=4) as ex:
    futs = [ex.submit(square, i) for i in range(5)]
    done_results = []
    for f in concurrent.futures.as_completed(futs, timeout=5):
        done_results.append(f.result())

assert len(done_results) == 5, f"as_completed yielded {len(done_results)} futures, expected 5"
assert sorted(done_results) == [0, 1, 4, 9, 16], f"as_completed results = {sorted(done_results)!r}"

print("as_completed_yields_all_futures OK")
