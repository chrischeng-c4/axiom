# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "map_applies_over_iterable"
# subject = "concurrent.futures.Executor.map"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.Executor.map: Executor.map applies the function to each element and yields results in input order: map(x*2, [1,2,3,4]) -> [2,4,6,8]"""
import concurrent.futures

with concurrent.futures.ThreadPoolExecutor(max_workers=2) as ex:
    results = list(ex.map(lambda x: x * 2, [1, 2, 3, 4], timeout=5))
    assert results == [2, 4, 6, 8], f"map results (in input order) = {results!r}"

print("map_applies_over_iterable OK")
