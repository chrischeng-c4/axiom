# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "wait_partitions_done_not_done"
# subject = "concurrent.futures.wait"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.wait: wait(..., return_when=ALL_COMPLETED) returns a (done, not_done) pair; with all five futures finished done has 5 and not_done is empty"""
import concurrent.futures


def identity(x):
    return x


with concurrent.futures.ThreadPoolExecutor(max_workers=4) as ex:
    futs = [ex.submit(identity, i) for i in range(5)]
    done, not_done = concurrent.futures.wait(
        futs, timeout=5, return_when=concurrent.futures.ALL_COMPLETED
    )
    assert len(done) == 5, f"all five futures done: {len(done)!r}"
    assert len(not_done) == 0, f"none pending: {len(not_done)!r}"

print("wait_partitions_done_not_done OK")
