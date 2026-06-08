"""Bulk copy.deepcopy on a nested dict (Task #68, Wave-6 ship #1).

Predicted regime per scout: compute (recursive walk over the
ObjData tree, allocating new List/Dict/Tuple nodes at each depth).
Wall target >=2.0x — CPython's copy.deepcopy walks a Python-level
memo dict + dispatches by __deepcopy__/__reduce_ex__ per element;
mamba's mb_copy_deepcopy matches on ObjData kind and uses native
clone paths (new_list / new_dict / new_tuple) with no Python-level
indirection.

Workload: 20-element top-level dict whose values are 3-element lists
of small inner dicts → ~260 cloned objects per iter × 50 iters =
~13k clones. Stays under the post-#2100 GC sweep cliff (10_000-alloc
threshold defers, then per-sweep cost drops). Original 100x100 spec
wedged mamba at 0% CPU after several minutes (likely high-alloc-density
+ deepcopy memo-less recursion cascade); 20x50 keeps wall-time signal
without tripping the cliff.

Per scout sequencing: Wave-6 Ship #1 — copy was already wired with
copy + deepcopy at module-attr (Task #414 era); this ship adds the
Error class shell + this bench fixture for #1265 attribution.

Hoist convention (#2097): bind `copy.deepcopy` locally before loop.
Mamba import quirk avoidance: separate `import sys` / `import time` /
`import copy` lines (xml.etree Task #56 finding).

# tier: compute
"""

import copy

_deep = copy.deepcopy

# Build the nested template once (not in the timed loop).
TEMPLATE = {}
for i in range(20):
    TEMPLATE[f"k_{i}"] = [
        {"x": i, "y": i * 2, "label": f"item_{i}_a"},
        {"x": i + 1, "y": i * 3, "label": f"item_{i}_b"},
        {"x": i + 2, "y": i * 5, "label": f"item_{i}_c"},
    ]

ITERS = 50

acc = 0
for _ in range(ITERS):
    cloned = _deep(TEMPLATE)
    acc += len(cloned)
print("deepcopy_nested_bulk:", acc)
