# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "security"
# case = "concurrent_mutation_stress"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = "shared-state SIGABRT under concurrent mutation (project_mamba_conformance_blockers)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: ~9 worker threads hammer a shared dict + list (insert/pop/slice-del/clear) while the main thread drives gc.collect() across the racing containers; the process must not crash or deadlock and both structures stay type-valid and usable afterward"""
import threading

import gc

WORKERS = 9
ITERS = 4000
JOIN_TIMEOUT = 5.0  # seconds; generous vs the < 2s deterministic CPython budget

shared_dict = {}
shared_list = []
stop = threading.Event()
errors = []  # worker exceptions are recorded, not swallowed


def _hammer(wid):
    try:
        for i in range(ITERS):
            if stop.is_set():
                break
            key = (wid << 20) | i
            shared_dict[key] = i
            shared_list.append(key)
            # Periodic pop / slice-del / clear forces shrink + free churn that
            # races the main thread's gc.collect() sweeps.
            if i % 3 == 0:
                shared_dict.pop(key, None)
            if i % 256 == 0:
                del shared_list[: len(shared_list) // 2]
            if i % 1000 == 0:
                shared_dict.clear()
    except Exception as exc:  # noqa: BLE001 - capture, never crash the run
        errors.append((wid, repr(exc)))


threads = [threading.Thread(target=_hammer, args=(w,)) for w in range(WORKERS)]
for t in threads:
    t.start()

# Main thread races the workers with repeated full collections.
for _ in range(200):
    gc.collect()

stop.set()

# Bounded join: a deadlock must show up as a still-alive thread, not a hang.
deadline_hit = False
for t in threads:
    t.join(timeout=JOIN_TIMEOUT)
    if t.is_alive():
        deadline_hit = True

assert not deadline_hit, "worker thread failed to join within timeout (deadlock?)"
print("all_joined:", not deadline_hit)
assert errors == [], f"worker(s) raised under concurrent mutation: {errors!r}"
print("no_worker_errors:", errors == [])

# Structures must remain valid, type-correct containers (no corruption / UAF).
assert isinstance(shared_dict, dict), f"shared_dict corrupted: {type(shared_dict)!r}"
assert isinstance(shared_list, list), f"shared_list corrupted: {type(shared_list)!r}"
print("dict_valid:", isinstance(shared_dict, dict))
print("list_valid:", isinstance(shared_list, list))

# The containers must still be usable after the storm (post-race mutation works
# and round-trips), proving internal invariants survived the GC races.
shared_dict["sentinel"] = 1
shared_list.append("sentinel")
assert shared_dict["sentinel"] == 1, "post-race dict insert failed"
assert shared_list[-1] == "sentinel", "post-race list append failed"
assert len(shared_dict) == len(dict(shared_dict)), "dict len/copy disagree (corruption)"
assert len(shared_list) == len(list(shared_list)), "list len/copy disagree (corruption)"
print("post_race_usable: True")
print("gc_collect_calls: 200")

print("concurrent_mutation_stress OK")
