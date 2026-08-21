# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "largefile"
# dimension = "behavior"
# case = "c_large_file_test__test_osstat"
# subject = "cpython.test_largefile.CLargeFileTest.test_osstat"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_largefile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_largefile.py::CLargeFileTest::test_osstat
"""Auto-ported test: CLargeFileTest::test_osstat (CPython 3.12 oracle)."""


import os
import stat
import tempfile


if os.environ.get("MAMBA_RUN_LARGEFILE") != "1":
    print("CLargeFileTest::test_osstat: skipped, set MAMBA_RUN_LARGEFILE=1 to run")
    raise SystemExit(0)

size = 2_500_000_000

with tempfile.TemporaryDirectory() as tmpdir:
    path = os.path.join(tmpdir, "largefile.bin")
    with open(path, "w+b") as handle:
        handle.write(b"z")
        handle.seek(size)
        handle.write(b"a")
        handle.flush()
        assert os.fstat(handle.fileno())[stat.ST_SIZE] == size + 1

    assert os.stat(path)[stat.ST_SIZE] == size + 1

print("CLargeFileTest::test_osstat: ok")
