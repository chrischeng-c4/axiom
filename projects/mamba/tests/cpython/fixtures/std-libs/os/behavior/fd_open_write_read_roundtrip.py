# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "fd_open_write_read_roundtrip"
# subject = "os.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.open: low-level os.open/os.write/os.lseek/os.read/os.close round-trips bytes (bytes, bytearray, memoryview) through a temp file; os.access reports R_OK and W_OK"""
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data")

    # os.open returns an int fd; O_CREAT|O_WRONLY creates a writable file.
    fd = os.open(path, os.O_CREAT | os.O_WRONLY)
    assert isinstance(fd, int), f"open fd type = {type(fd)!r}"

    # os.write accepts bytes, bytearray, and memoryview.
    n = os.write(fd, b"bacon\n")
    assert n == 6, f"write returned {n!r}"
    os.write(fd, bytearray(b"eggs\n"))
    os.write(fd, memoryview(b"spam\n"))
    os.close(fd)

    # os.access reports the file is readable and writable.
    assert os.access(path, os.R_OK), "file should be readable"
    assert os.access(path, os.W_OK), "file should be writable"

    # os.open + os.lseek + os.read round-trips the written bytes.
    rfd = os.open(path, os.O_RDONLY)
    os.lseek(rfd, 0, os.SEEK_SET)
    chunk = os.read(rfd, 6)
    assert type(chunk) is bytes, f"read type = {type(chunk)!r}"
    assert chunk == b"bacon\n", f"read = {chunk!r}"
    os.close(rfd)

    # Whole-file contents split into lines.
    with open(path, "rb") as fobj:
        lines = fobj.read().splitlines()
    assert lines == [b"bacon", b"eggs", b"spam"], f"lines = {lines!r}"
print("fd_open_write_read_roundtrip OK")
