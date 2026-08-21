# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "behavior"
# case = "array_shared_across_process"
# subject = "multiprocessing.Array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/_test_multiprocessing.py"
# status = "filled"
# ///
"""multiprocessing.Array: an Array('i', [1,2,3,4,5]) is shared memory; a child does arr[2] += 10 and after join the parent observes arr[2] == 13 (spawn-guarded under __main__)"""
import multiprocessing


def _increment_shared(arr, idx):
    arr[idx] += 10


if __name__ == "__main__":
    arr = multiprocessing.Array("i", [1, 2, 3, 4, 5])
    p = multiprocessing.Process(target=_increment_shared, args=(arr, 2))
    p.start()
    p.join(timeout=10)
    assert arr[2] == 13, f"shared Array[2] = {arr[2]!r}"

    print("array_shared_across_process OK")
