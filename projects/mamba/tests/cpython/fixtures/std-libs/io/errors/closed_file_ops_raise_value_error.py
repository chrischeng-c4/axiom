# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "errors"
# case = "closed_file_ops_raise_value_error"
# subject = "io"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io: every operation (flush/read/write/seek/tell/truncate/next/...) on a closed file object raises ValueError, across text, buffered and raw layers"""
import io

import os
import tempfile

MODES = [
    {"mode": "w"}, {"mode": "wb"},
    {"mode": "w", "buffering": 1}, {"mode": "wb", "buffering": 0},
    {"mode": "r"}, {"mode": "rb"}, {"mode": "rb", "buffering": 0},
    {"mode": "w+"}, {"mode": "w+b"}, {"mode": "w+b", "buffering": 0},
]


def expect_value_error(fn, what):
    try:
        fn()
    except ValueError:
        return
    raise AssertionError(f"{what}: expected ValueError on closed stream")


with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    for kwargs in MODES:
        binary = "b" in kwargs["mode"]
        if not binary:
            kwargs = {**kwargs, "encoding": "utf-8"}
        f = open(path, **kwargs)
        f.close()

        expect_value_error(f.flush, "flush")
        expect_value_error(f.fileno, "fileno")
        expect_value_error(f.isatty, "isatty")
        expect_value_error(f.__iter__, "__iter__")
        expect_value_error(f.read, "read")
        expect_value_error(f.readline, "readline")
        expect_value_error(f.readlines, "readlines")
        expect_value_error(lambda: f.seek(0), "seek")
        expect_value_error(f.tell, "tell")
        expect_value_error(f.truncate, "truncate")
        expect_value_error(lambda: f.write(b"" if binary else ""), "write")
        expect_value_error(lambda: f.writelines([]), "writelines")
        expect_value_error(lambda: next(f), "next")
        if hasattr(f, "peek"):
            expect_value_error(lambda: f.peek(1), "peek")
        if hasattr(f, "readinto"):
            expect_value_error(lambda: f.readinto(bytearray(8)), "readinto")

print("closed_file_ops_raise_value_error OK")
