"""Bulk glob.glob over a ~50-file dir (Task #66, Wave-5 ship #4).

Predicted regime per scout: compute + I/O. Per-iter cost is one
readdir() syscall + N filename-pattern matches. mamba uses
std::fs::read_dir + Rust-side glob_match (linear two-pointer star
algorithm), CPython uses os.scandir + fnmatch.fnmatchcase. Wall
target >=1.5x — I/O is roughly equal, mamba wins on the matching
loop only.

Workload: tmpdir with 50 files (25 .txt, 25 .rs) × `*.txt` × 100 iters.

Setup happens ONCE outside the timing window. Hot path is just
`glob.glob(pattern)` with the pattern hoisted. The setup uses
`tempfile.mkdtemp` and writes empty files via `with open(...) as fh`
— both verified callable on mamba (tempfile_basic.py + linecache).

Hoist convention (#2097): bind `glob.glob` and the pattern string
locally before the loop. Mamba import quirk: separate `import` lines
for sys/time/glob/tempfile/os/shutil (comma form only binds first,
xml.etree Task #56).

# tier: compute_io
"""

import glob
import tempfile
import os
import shutil

_glob = glob.glob

_tmp = tempfile.mkdtemp(prefix="mb_glob_bench_")
for i in range(25):
    path = os.path.join(_tmp, f"file_{i}.txt")
    with open(path, "w") as fh:
        fh.write("")
for i in range(25):
    path = os.path.join(_tmp, f"file_{i}.rs")
    with open(path, "w") as fh:
        fh.write("")

PAT = os.path.join(_tmp, "*.txt")
ITERS = 1000

acc = 0
for _ in range(ITERS):
    matched = _glob(PAT)
    acc += len(matched)
print("glob_txt_bulk:", acc)

shutil.rmtree(_tmp, ignore_errors=True)
