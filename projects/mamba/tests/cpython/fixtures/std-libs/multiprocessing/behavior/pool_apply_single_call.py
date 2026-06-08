# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "behavior"
# case = "pool_apply_single_call"
# subject = "multiprocessing.Pool"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/_test_multiprocessing.py"
# status = "filled"
# ///
"""multiprocessing.Pool: Pool(processes=1).apply runs a single module-level call in a worker and returns its value: apply(square, (7,)) == 49 (spawn-guarded under __main__)"""
import multiprocessing


def _square(x):
    return x * x


if __name__ == "__main__":
    with multiprocessing.Pool(processes=1) as pool:
        r = pool.apply(_square, (7,))
    assert r == 49, f"pool.apply = {r!r}"

    print("pool_apply_single_call OK")
