# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "behavior"
# case = "workers_one_compiles_serially"
# subject = "compileall.compile_dir"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compileall.py"
# status = "filled"
# ///
"""compileall.compile_dir: compile_dir with workers=1 compiles every file in-process (serial, no process pool) and produces one .pyc per source"""
import compileall
import os
import tempfile

# workers=1 stays in-process (no ProcessPoolExecutor, which would be
# nondeterministic and unsafe to spawn from a bare script).
with tempfile.TemporaryDirectory() as d:
    for name in ("a.py", "b.py", "c.py"):
        with open(os.path.join(d, name), "w") as f:
            f.write("v = 1\n")
    ok = compileall.compile_dir(d, quiet=2, workers=1)
    assert ok, "workers=1 compiles"
    cache = os.path.join(d, "__pycache__")
    pycs = [f for f in os.listdir(cache) if f.endswith(".pyc")]
    assert len(pycs) == 3, pycs
print("workers_one_compiles_serially OK")
