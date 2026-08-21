# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bufio"
# dimension = "behavior"
# case = "buffer_size_test__test_primepat"
# subject = "cpython.test_bufio.BufferSizeTest.test_primepat"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bufio.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: BufferSizeTest::test_primepat (CPython 3.12 oracle)."""

import _pyio as pyio
import io
import tempfile


LENGTHS = (
    list(range(1, 257))
    + [512, 1000, 1024, 2048, 4096, 8192, 10000, 16384, 32768, 65536, 1000000]
)


def try_one(open_func, path, payload):
    with open_func(path, "wb") as f:
        f.write(payload)
        f.write(b"\n")
        f.write(payload)

    with open_func(path, "rb") as f:
        assert f.readline() == payload + b"\n"
        assert f.readline() == payload
        assert not f.readline()


def drive_one(open_func, pattern):
    with tempfile.TemporaryDirectory() as tmp:
        path = f"{tmp}/bufio.bin"
        for length in LENGTHS:
            q, r = divmod(length, len(pattern))
            teststring = pattern * q + pattern[:r]
            assert len(teststring) == length
            try_one(open_func, path, teststring)
            try_one(open_func, path, teststring + b"x")
            try_one(open_func, path, teststring[:-1])


for open_func in (io.open, pyio.open):
    drive_one(open_func, b"1234567890\00\01\02\03\04\05\06")

print("BufferSizeTest::test_primepat: ok")
