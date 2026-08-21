"""Behavior contract for third-party aiofiles package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import aiofiles  # type: ignore[import]
import asyncio
import tempfile
import os

# Rule 1: aiofiles.open returns an async context manager
async def _rule1():
    with tempfile.NamedTemporaryFile(mode="w", suffix=".txt",
                                     delete=False) as _f:
        _path1 = _f.name
        _f.write("hello aiofiles")
    try:
        async with aiofiles.open(_path1, "r") as _af:
            _content1 = await _af.read()
        assert _content1 == "hello aiofiles", f"read = {_content1!r}"
    finally:
        os.unlink(_path1)

asyncio.run(_rule1())

# Rule 2: aiofiles.open write then read round-trip
async def _rule2():
    with tempfile.NamedTemporaryFile(suffix=".txt", delete=False) as _f:
        _path2 = _f.name
    try:
        async with aiofiles.open(_path2, "w") as _af2:
            await _af2.write("async write test")
        async with aiofiles.open(_path2, "r") as _af2r:
            _data2 = await _af2r.read()
        assert _data2 == "async write test", f"round-trip = {_data2!r}"
    finally:
        os.unlink(_path2)

asyncio.run(_rule2())

# Rule 3: aiofiles.open binary mode
async def _rule3():
    with tempfile.NamedTemporaryFile(suffix=".bin", delete=False) as _f:
        _path3 = _f.name
    try:
        _bytes3 = b"\x00\x01\x02\x03"
        async with aiofiles.open(_path3, "wb") as _af3:
            await _af3.write(_bytes3)
        async with aiofiles.open(_path3, "rb") as _af3r:
            _data3 = await _af3r.read()
        assert _data3 == _bytes3, f"binary round-trip = {_data3!r}"
    finally:
        os.unlink(_path3)

asyncio.run(_rule3())

# Rule 4: aiofiles.open readline
async def _rule4():
    with tempfile.NamedTemporaryFile(mode="w", suffix=".txt",
                                     delete=False) as _f:
        _path4 = _f.name
        _f.write("line1\nline2\nline3\n")
    try:
        async with aiofiles.open(_path4, "r") as _af4:
            _line1 = await _af4.readline()
            _line2 = await _af4.readline()
        assert _line1 == "line1\n", f"line1 = {_line1!r}"
        assert _line2 == "line2\n", f"line2 = {_line2!r}"
    finally:
        os.unlink(_path4)

asyncio.run(_rule4())

# Rule 5: Module attributes are identity-stable
_open_ref = aiofiles.open
_tf_ref = aiofiles.tempfile
_si_ref = aiofiles.stdin
_so_ref = aiofiles.stdout
for _ in range(5):
    assert aiofiles.open is _open_ref, "open stable"
    assert aiofiles.tempfile is _tf_ref, "tempfile stable"
    assert aiofiles.stdin is _si_ref, "stdin stable"
    assert aiofiles.stdout is _so_ref, "stdout stable"

print("behavior OK")
