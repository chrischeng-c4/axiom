"""Bulk csv.reader iterate-then-sum hot loop (Task #53, Wave-4 ship #2).

Predicted regime per scout: compute for tokenize, balanced overall.
Realistic CSV workloads are 100-row-class chunks parsed many times
(per-request log line, per-row config dump, etc.) — NOT 10k-row
bulk parse, which is a CPython C-state-machine specialty and beyond
mamba's batch-construction shim (subset-B per-call at extreme
density).

Workload: 100-row x 5-col CSV, parsed 500 times. Same total work
(50k row-parses, 250k field-strs) but distributed across many
small calls, matching the realistic-CSV regime the scout predicted.

No #2100 callbacks. No #2129 operator overloads. No bulk-bytes
materialization (input is module-level text).

Hoist convention (#2097): bind `csv.reader` to a local before the
loop. CPython's `csv.reader` expects an iterable of lines (not a
single string), so we feed the pre-split list[str]. Mamba's
`mb_csv_reader` accepts either form.

# tier: compute
"""

import csv

_reader = csv.reader

LINES = [",".join([f"v{i}" for i in range(5)]) for _ in range(100)]
ITERS = 500

acc = 0
for _ in range(ITERS):
    rows = _reader(LINES)
    for row in rows:
        acc += len(row)
print("reader_rows:", acc)
