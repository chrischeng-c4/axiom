# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "real_world"
# case = "pool_map_reduce_pipeline"
# subject = "multiprocessing.Pool"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""multiprocessing.Pool: a batch job fans a CPU-light transform over a work list with a process Pool (map + imap_unordered), reduces the per-item results to a deterministic total, and shows the pool drains cleanly on context exit (spawn-guarded under __main__)"""
import multiprocessing


# A deterministic CPU-light transform standing in for a per-record job.
# Module-level so the spawn start method can pickle it for each worker.
def _transform(n):
    return n * n + 1


if __name__ == "__main__":
    work = list(range(64))
    expected_total = sum(_transform(n) for n in work)

    # --- Stage 1: fan out with Pool.map, which preserves input order. -------
    with multiprocessing.Pool(processes=4) as pool:
        mapped = pool.map(_transform, work)
    assert mapped == [_transform(n) for n in work], "map preserves input order"
    map_total = sum(mapped)
    assert map_total == expected_total, f"map reduce = {map_total}, expected {expected_total}"

    # --- Stage 2: the same transform via imap_unordered, reduced. -----------
    # A fresh pool is needed because the stage-1 pool drained on context exit.
    with multiprocessing.Pool(processes=4) as pool:
        unordered_total = sum(pool.imap_unordered(_transform, work))
    assert unordered_total == expected_total, (
        f"imap_unordered reduce = {unordered_total}, expected {expected_total}"
    )

    # --- Stage 3: both fan-out strategies agree on the reduced total. -------
    assert map_total == unordered_total, "map vs imap_unordered reduce disagree"

    print("pool_map_reduce_pipeline OK:", map_total)
