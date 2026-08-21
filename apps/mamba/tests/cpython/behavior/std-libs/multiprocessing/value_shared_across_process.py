# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "behavior"
# case = "value_shared_across_process"
# subject = "multiprocessing.Value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/_test_multiprocessing.py"
# status = "filled"
# ///
"""multiprocessing.Value: a Value('i', 0) is shared memory; a child sets .value = 99 and after join the parent observes 99 (spawn-guarded under __main__)"""
import multiprocessing


def _set_value(v, n):
    v.value = n


if __name__ == "__main__":
    val = multiprocessing.Value("i", 0)
    p = multiprocessing.Process(target=_set_value, args=(val, 99))
    p.start()
    p.join(timeout=10)
    assert val.value == 99, f"shared Value = {val.value!r}"

    print("value_shared_across_process OK")
