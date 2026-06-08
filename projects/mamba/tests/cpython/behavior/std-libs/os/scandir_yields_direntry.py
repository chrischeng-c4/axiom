# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "scandir_yields_direntry"
# subject = "os.scandir"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.scandir: os.scandir yields os.DirEntry objects exposing name/path and is_file()/is_dir() predicates over a temp dir containing a file and a subdir"""
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    # Populate a file and a subdirectory.
    fpath = os.path.join(d, "file.txt")
    with open(fpath, "w", encoding="utf-8") as f:
        f.write("python")
    os.mkdir(os.path.join(d, "subdir"))

    # scandir yields os.DirEntry objects exposing name/path/type predicates.
    by_name = {}
    with os.scandir(d) as it:
        for entry in it:
            assert isinstance(entry, os.DirEntry), f"not DirEntry: {entry!r}"
            by_name[entry.name] = entry

    assert set(by_name) == {"file.txt", "subdir"}, f"names = {set(by_name)!r}"
    assert by_name["file.txt"].is_file(), "file.txt is_file"
    assert not by_name["file.txt"].is_dir(), "file.txt not is_dir"
    assert by_name["subdir"].is_dir(), "subdir is_dir"
    assert by_name["file.txt"].path == fpath, f"path = {by_name['file.txt'].path!r}"
print("scandir_yields_direntry OK")
