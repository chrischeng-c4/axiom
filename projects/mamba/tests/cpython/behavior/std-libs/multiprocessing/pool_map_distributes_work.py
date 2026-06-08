# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "behavior"
# case = "pool_map_distributes_work"
# subject = "multiprocessing.Pool"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/_test_multiprocessing.py"
# status = "filled"
# ///
"""multiprocessing.Pool: Pool(processes=2).map applies a module-level square fn over [1,2,3,4,5] across worker processes and returns [1,4,9,16,25] in input order (spawn-guarded under __main__)"""
import multiprocessing


# Pool work fn must be a picklable module-level function under spawn.
def _square(x):
    return x * x


if __name__ == "__main__":
    with multiprocessing.Pool(processes=2) as pool:
        results = pool.map(_square, [1, 2, 3, 4, 5])
    assert results == [1, 4, 9, 16, 25], f"pool.map = {results!r}"

    print("pool_map_distributes_work OK")
