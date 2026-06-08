# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_pathlib_zipfile_tarfile_calendar_silent"
# subject = "cpython321.lang_pathlib_zipfile_tarfile_calendar_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_pathlib_zipfile_tarfile_calendar_silent.py"
# status = "filled"
# ///
"""cpython321.lang_pathlib_zipfile_tarfile_calendar_silent: execute CPython 3.12 seed lang_pathlib_zipfile_tarfile_calendar_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `str(pathlib.PurePath('/a/b'))` (the
# documented "PurePath str returns the canonical path string '/a/b'"
# — mamba returns '<PurePosixPath instance>'), `pathlib.PurePath
# ('/a/b').name` (the documented "PurePath.name returns the final
# component — returns 'b'" — mamba returns None), `pathlib.PurePath
# ('/a/b.txt').suffix` (the documented "PurePath.suffix returns the
# extension — returns '.txt'" — mamba returns None), `pathlib.
# PurePath('/a/b').parts` (the documented "PurePath.parts returns
# the (root, *components) tuple — returns ('/', 'a', 'b')" — mamba
# returns None), `hasattr(zipfile, 'ZipInfo')` (the documented
# "zipfile exposes the ZipInfo class" — mamba returns False),
# `hasattr(zipfile, 'BadZipFile')` (the documented "zipfile exposes
# the BadZipFile exception" — mamba returns False), `hasattr
# (tarfile, 'TarFile')` (the documented "tarfile exposes the TarFile
# class" — mamba returns False), `hasattr(tarfile, 'TarError')` (the
# documented "tarfile exposes the TarError exception" — mamba
# returns False), `hasattr(tarfile, 'USTAR_FORMAT')` (the
# documented "tarfile exposes the USTAR_FORMAT constant" — mamba
# returns False), and `tarfile.USTAR_FORMAT == 0` (the documented
# "USTAR_FORMAT enum value is 0" — mamba returns None).
# Ten-pack pinned to atomic 276.
#
# Behavioral edges that CONFORM on mamba (pathlib — hasattr Path/
# PurePath/PurePosixPath/PureWindowsPath/PosixPath/WindowsPath.
# zipfile — hasattr ZipFile/ZIP_STORED/ZIP_DEFLATED/is_zipfile +
# ZIP_STORED==0 / ZIP_DEFLATED==8. tarfile — hasattr open/
# is_tarfile. calendar — hasattr Calendar/TextCalendar/HTMLCalendar/
# monthrange/isleap/weekday/calendar/month/timegm/leapdays/month_
# name/day_name/month_abbr/day_abbr/MONDAY/TUESDAY/SUNDAY + isleap
# 2020/2000 True / 2021/1900 False + monthrange 2020/2021 Feb +
# weekday 2020-01-01 == 2 + MONDAY==0 / SUNDAY==6 + leapdays
# 2000..2020 == 5) are covered in the matching pass fixture
# `test_pathlib_zipfile_tarfile_calendar_value_ops`.
import pathlib
import zipfile
import tarfile


_ledger: list[int] = []

# 1) str(pathlib.PurePath('/a/b')) — canonical path str
#    (mamba: returns '<PurePosixPath instance>')
assert str(pathlib.PurePath("/a/b")) == "/a/b"; _ledger.append(1)

# 2) PurePath('/a/b').name — final component
#    (mamba: returns None)
assert pathlib.PurePath("/a/b").name == "b"; _ledger.append(1)

# 3) PurePath('/a/b.txt').suffix — extension
#    (mamba: returns None)
assert pathlib.PurePath("/a/b.txt").suffix == ".txt"; _ledger.append(1)

# 4) PurePath('/a/b').parts — (root, *components) tuple
#    (mamba: returns None)
assert pathlib.PurePath("/a/b").parts == ("/", "a", "b"); _ledger.append(1)

# 5) hasattr(zipfile, 'ZipInfo') — ZipInfo class
#    (mamba: returns False)
assert hasattr(zipfile, "ZipInfo") == True; _ledger.append(1)

# 6) hasattr(zipfile, 'BadZipFile') — exception class
#    (mamba: returns False)
assert hasattr(zipfile, "BadZipFile") == True; _ledger.append(1)

# 7) hasattr(tarfile, 'TarFile') — TarFile class
#    (mamba: returns False)
assert hasattr(tarfile, "TarFile") == True; _ledger.append(1)

# 8) hasattr(tarfile, 'TarError') — exception class
#    (mamba: returns False)
assert hasattr(tarfile, "TarError") == True; _ledger.append(1)

# 9) hasattr(tarfile, 'USTAR_FORMAT') — format constant
#    (mamba: returns False)
assert hasattr(tarfile, "USTAR_FORMAT") == True; _ledger.append(1)

# 10) tarfile.USTAR_FORMAT == 0 — format enum value
#     (mamba: returns None)
assert tarfile.USTAR_FORMAT == 0; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_pathlib_zipfile_tarfile_calendar_silent {sum(_ledger)} asserts")
