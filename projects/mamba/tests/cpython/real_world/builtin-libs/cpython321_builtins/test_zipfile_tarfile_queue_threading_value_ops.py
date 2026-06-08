# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_zipfile_tarfile_queue_threading_value_ops"
# subject = "cpython321.test_zipfile_tarfile_queue_threading_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_zipfile_tarfile_queue_threading_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_zipfile_tarfile_queue_threading_value_ops: execute CPython 3.12 seed test_zipfile_tarfile_queue_threading_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of four
# storage / concurrency stdlib modules used by every archive
# pipeline and worker pool: `zipfile` (the ZIP_STORED /
# ZIP_DEFLATED compression-method integer sentinels + the
# is_zipfile inspector), `tarfile` (the is_tarfile inspector),
# `sqlite3` (the PARSE_DECLTYPES / PARSE_COLNAMES type-detection
# integer sentinels), `queue` (Queue lifecycle — put / get /
# qsize / empty), and `threading` (active_count / current_thread /
# main_thread / get_ident returning an int).
#
# The matching subset between mamba and CPython is the byte-exact
# return-value layer: zipfile.ZIP_STORED == 0 and ZIP_DEFLATED == 8
# (documented compression-method codes); zipfile.is_zipfile and
# tarfile.is_tarfile return False for any non-archive file;
# sqlite3.PARSE_DECLTYPES == 1 and PARSE_COLNAMES == 2 (documented
# detect_types integer flags); queue.Queue() supports the
# documented put / get / qsize / empty lifecycle on a single
# thread; threading.active_count returns 1 in a single-threaded
# process, threading.current_thread / main_thread return non-None
# objects, threading.get_ident returns an `int`.
#
# Surface in this fixture:
#   • zipfile.ZIP_STORED == 0 (no compression);
#   • zipfile.ZIP_DEFLATED == 8 (deflate compression);
#   • zipfile.is_zipfile("/etc/hosts") == False (a text file is
#     not a ZIP archive);
#   • tarfile.is_tarfile("/etc/hosts") == False (a text file is
#     not a tar archive);
#   • sqlite3.PARSE_DECLTYPES == 1 (parse declared types flag);
#   • sqlite3.PARSE_COLNAMES == 2 (parse column-name flag);
#   • queue.Queue().put / get / qsize / empty lifecycle matches
#     the documented FIFO ordering;
#   • threading.active_count() == 1 (single-thread process);
#   • threading.current_thread() / threading.main_thread() return
#     non-None values;
#   • type(threading.get_ident()).__name__ == "int" — thread ID is
#     a plain integer.
#
# Behavioral edges that DIVERGE on mamba (zipfile.ZipFile /
# BadZipFile class identity, zipfile.ZIP_BZIP2 / ZIP_LZMA integer
# sentinels, tarfile.TarFile class identity, tarfile.REGTYPE /
# DIRTYPE / LNKTYPE / SYMTYPE byte sentinels, tarfile.GNU_FORMAT
# / PAX_FORMAT / USTAR_FORMAT / DEFAULT_FORMAT integer sentinels,
# sqlite3.sqlite_version / paramstyle / threadsafety module-level
# constants, sqlite3.Connection / Cursor / Row / Error class
# identity, sqlite3.connect lifecycle, threading.Thread / Lock
# class identity, type(threading.current_thread()) == _MainThread
# class identity, queue.Queue / LifoQueue / PriorityQueue /
# SimpleQueue / Empty / Full class identity, type(queue.Queue())
# == "Queue") are covered in `lang_sqlite3_zipfile_tarfile_queue_
# class_silent`.
import zipfile
import tarfile
import sqlite3
import queue
import threading

_ledger: list[int] = []

# 1) zipfile — compression-method integer sentinels
assert zipfile.ZIP_STORED == 0; _ledger.append(1)
assert zipfile.ZIP_DEFLATED == 8; _ledger.append(1)

# 2) zipfile.is_zipfile — text file is not an archive
assert zipfile.is_zipfile("/etc/hosts") == False; _ledger.append(1)

# 3) tarfile.is_tarfile — text file is not an archive
assert tarfile.is_tarfile("/etc/hosts") == False; _ledger.append(1)

# 4) sqlite3 — detect_types integer flags
assert sqlite3.PARSE_DECLTYPES == 1; _ledger.append(1)
assert sqlite3.PARSE_COLNAMES == 2; _ledger.append(1)

# 5) queue.Queue — single-thread put / get / qsize / empty
_q = queue.Queue()
_q.put(1)
_q.put(2)
assert _q.qsize() == 2; _ledger.append(1)
assert _q.empty() == False; _ledger.append(1)
assert _q.get() == 1; _ledger.append(1)
assert _q.get() == 2; _ledger.append(1)
assert _q.empty() == True; _ledger.append(1)

# 6) queue.Queue — FIFO ordering preserved
_q2 = queue.Queue()
_q2.put("a")
_q2.put("b")
_q2.put("c")
assert _q2.get() == "a"; _ledger.append(1)
assert _q2.get() == "b"; _ledger.append(1)
assert _q2.get() == "c"; _ledger.append(1)

# 7) threading.active_count — single-thread process
assert threading.active_count() == 1; _ledger.append(1)

# 8) threading.current_thread / main_thread — return non-None
assert threading.current_thread() is not None; _ledger.append(1)
assert threading.main_thread() is not None; _ledger.append(1)

# 9) threading.get_ident — thread ID is a plain int
assert isinstance(threading.get_ident(), int); _ledger.append(1)
assert threading.get_ident() > 0; _ledger.append(1)

# 10) hasattr surface — module-level helpers
assert hasattr(zipfile, "ZipFile"); _ledger.append(1)
assert hasattr(zipfile, "is_zipfile"); _ledger.append(1)
assert hasattr(zipfile, "ZIP_STORED"); _ledger.append(1)
assert hasattr(zipfile, "ZIP_DEFLATED"); _ledger.append(1)
assert hasattr(tarfile, "is_tarfile"); _ledger.append(1)
assert hasattr(sqlite3, "PARSE_DECLTYPES"); _ledger.append(1)
assert hasattr(sqlite3, "PARSE_COLNAMES"); _ledger.append(1)
assert hasattr(queue, "Queue"); _ledger.append(1)
assert hasattr(threading, "active_count"); _ledger.append(1)
assert hasattr(threading, "current_thread"); _ledger.append(1)
assert hasattr(threading, "main_thread"); _ledger.append(1)
assert hasattr(threading, "get_ident"); _ledger.append(1)

# NB: zipfile.ZipFile / BadZipFile, zipfile.ZIP_BZIP2 / ZIP_LZMA,
# tarfile.TarFile, tarfile.REGTYPE / DIRTYPE / LNKTYPE / SYMTYPE,
# tarfile.GNU_FORMAT / PAX_FORMAT / USTAR_FORMAT / DEFAULT_FORMAT,
# sqlite3 sqlite_version / paramstyle / threadsafety, sqlite3
# Connection / Cursor / Row / Error class identity, sqlite3.connect
# lifecycle, threading.Thread / Lock class identity, type(threading.
# current_thread()) == _MainThread, queue.Queue / LifoQueue /
# PriorityQueue / SimpleQueue / Empty / Full class identity all
# DIVERGE on mamba — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_zipfile_tarfile_queue_threading_value_ops {sum(_ledger)} asserts")
