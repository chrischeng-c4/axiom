# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "readiness"
# lib = "threading"
# dimension = "concurrency"
# case = "container_mutation_rounds"
# subject = "list.append,dict.__setitem__,set.add"
# kind = "readiness"
# max_wall_s = 120
# mem_carveout = ""
# source = "issue #4244"
# status = "filled"
# ///
"""Concurrency readiness: repeated rounds of threaded container mutation
must not corrupt counts, and the containers a completed round drops must
not be retained by the next round.

Each round starts four threads; each thread performs 2000 `list.append`,
2000 `dict.__setitem__` (distinct keys), and 2000 `set.add` (distinct
values) into a list, dict, and set created fresh for that round inside
`run_fixture`, then all four threads are joined and each container's count
is checked against the expected 4 * 2000 = 8000. The round count comes from
`MAMBA_FIXTURE_ROUNDS` so the harness can run this file once at 5 rounds and
again at 50 rounds and compare the two runs' peak RSS -- the correctness
half of the contract this file prints (`readiness: PASS` /
`readiness: FAIL: <detail>`); the memory half is measured by the harness
reading the child's `rusage`, never anything this program prints.

Every accumulating loop stays inside a function -- a module-scope loop in a
module that also defines a function was issue #4242's defect -- so the round
loop lives inside `run_fixture`, called once (not in a loop) at module
scope. No `gc.collect()` and no explicit release call: this is plain Python
that CPython 3.12 runs unchanged.
"""
import os
import threading

N_THREADS = 4
PER_THREAD = 2000


def worker(idx: int, lst: list, dct: dict, st: set) -> None:
    for i in range(PER_THREAD):
        lst.append(i)
        dct[idx * PER_THREAD + i] = i
        st.add(idx * PER_THREAD + i)


def run_fixture() -> None:
    expected = N_THREADS * PER_THREAD
    rounds = int(os.environ["MAMBA_FIXTURE_ROUNDS"])
    failures: list = []
    for round_idx in range(rounds):
        lst: list = []
        dct: dict = {}
        st: set = set()
        threads = [
            threading.Thread(target=worker, args=(t, lst, dct, st))
            for t in range(N_THREADS)
        ]
        for t in threads:
            t.start()
        for t in threads:
            t.join()
        list_len = len(lst)
        dict_len = len(dct)
        set_len = len(st)
        ok = list_len == expected and dict_len == expected and set_len == expected
        print(
            f"round={round_idx} list={list_len} dict={dict_len} set={set_len} ok={ok}"
        )
        if not ok:
            failures.append(
                f"round {round_idx}: list={list_len} dict={dict_len} "
                f"set={set_len} expected={expected}"
            )
    if failures:
        print("readiness: FAIL: " + "; ".join(failures))
    else:
        print("readiness: PASS")


run_fixture()
