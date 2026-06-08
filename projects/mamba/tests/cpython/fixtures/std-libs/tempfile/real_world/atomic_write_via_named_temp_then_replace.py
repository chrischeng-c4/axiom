# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "real_world"
# case = "atomic_write_via_named_temp_then_replace"
# subject = "tempfile.NamedTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.NamedTemporaryFile: atomic-write pattern: write the new content to a NamedTemporaryFile(delete=False) in the target dir, os.replace it over the destination, and read the destination back to confirm the swap"""
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    dest = os.path.join(d, "config.txt")
    with open(dest, "w") as _f:
        _f.write("old contents")
    # Write the new contents to a sibling temp file, then atomically replace.
    tmp = tempfile.NamedTemporaryFile(mode="w", dir=d, delete=False)
    try:
        tmp.write("new contents")
        tmp.flush()
        os.fsync(tmp.fileno())
    finally:
        tmp.close()
    os.replace(tmp.name, dest)
    with open(dest) as _f:
        assert _f.read() == "new contents", "atomic replace swapped the file"
    assert os.listdir(d) == ["config.txt"], "no temp file left behind"
print("atomic_write_via_named_temp_then_replace OK")
