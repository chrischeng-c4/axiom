# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "real_world"
# case = "classify_real_file_from_os_stat"
# subject = "stat.S_ISREG"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_stat.py"
# status = "filled"
# ///
"""stat.S_ISREG: the canonical end-user flow: create a real temp file, call os.stat, and feed st_mode through S_ISREG / S_ISDIR / S_IMODE / filemode to classify it as a regular file"""
import os
import stat
import tempfile

with tempfile.TemporaryDirectory() as tmp:
    path = os.path.join(tmp, "sample.txt")
    with open(path, "w") as f:
        f.write("payload")
    os.chmod(path, 0o644)

    mode = os.stat(path).st_mode

    # The mode is recognized as a regular file, not a directory.
    assert stat.S_ISREG(mode) is True, "S_ISREG(real file)"
    assert stat.S_ISDIR(mode) is False, "S_ISDIR(real file)"

    # S_IMODE recovers the permission bits we set.
    assert stat.S_IMODE(mode) == 0o644, oct(stat.S_IMODE(mode))

    # filemode renders an ls -l style string for a regular file.
    rendered = stat.filemode(mode)
    assert rendered[0] == "-", rendered

    # The directory itself is recognized as a directory.
    dir_mode = os.stat(tmp).st_mode
    assert stat.S_ISDIR(dir_mode) is True, "S_ISDIR(tmpdir)"
    assert stat.S_ISREG(dir_mode) is False, "S_ISREG(tmpdir)"

print("classify_real_file_from_os_stat OK")
