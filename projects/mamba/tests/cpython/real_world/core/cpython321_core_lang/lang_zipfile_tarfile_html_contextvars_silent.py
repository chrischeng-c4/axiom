# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_zipfile_tarfile_html_contextvars_silent"
# subject = "cpython321.lang_zipfile_tarfile_html_contextvars_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_zipfile_tarfile_html_contextvars_silent.py"
# status = "filled"
# ///
"""cpython321.lang_zipfile_tarfile_html_contextvars_silent: execute CPython 3.12 seed lang_zipfile_tarfile_html_contextvars_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(zipfile, 'ZipInfo')` (the
# documented "zipfile exposes the ZipInfo metadata record" — mamba
# returns False), `hasattr(zipfile, 'BadZipFile')` (the documented
# "zipfile exposes the BadZipFile exception" — mamba returns False),
# `hasattr(zipfile, 'ZIP_BZIP2')` (the documented "zipfile exposes the
# ZIP_BZIP2 compression-method constant" — mamba returns False), `
# hasattr(tarfile, 'TarFile')` (the documented "tarfile exposes the
# TarFile class" — mamba returns False), `hasattr(tarfile, 'TarInfo')
# ` (the documented "tarfile exposes the TarInfo metadata record" —
# mamba returns False), `hasattr(tarfile, 'TarError')` (the documented
# "tarfile exposes the TarError exception" — mamba returns False), `
# hasattr(tarfile, 'USTAR_FORMAT')` (the documented "tarfile exposes
# the USTAR_FORMAT constant" — mamba returns False), `hasattr(html.
# entities, 'name2codepoint')` (the documented "html.entities exposes
# the name2codepoint dict" — mamba returns False), `hasattr(html.
# entities, 'codepoint2name')` (the documented "html.entities exposes
# the codepoint2name dict" — mamba returns False), and `type(
# contextvars.ContextVar('x')).__name__ == 'ContextVar'` (the
# documented "contextvars.ContextVar(name) returns a ContextVar
# instance" — mamba returns 'str' — constructor degrades to the name
# string).
# Ten-pack pinned to atomic 309.
#
# Behavioral edges that CONFORM on mamba (subprocess — hasattr Popen/
# run/call/check_call/check_output/PIPE/STDOUT/DEVNULL/CalledProcess
# Error/TimeoutExpired/SubprocessError/CompletedProcess + PIPE == -1 +
# STDOUT == -2 + DEVNULL == -3. selectors — hasattr DefaultSelector/
# SelectSelector/PollSelector/KqueueSelector/EVENT_READ/EVENT_WRITE/
# SelectorKey/BaseSelector + EVENT_READ == 1 + EVENT_WRITE == 2.
# zipfile — hasattr ZipFile/is_zipfile/ZIP_STORED/ZIP_DEFLATED + ZIP_
# STORED == 0 + ZIP_DEFLATED == 8. tarfile — hasattr open/is_tarfile.
# html.parser — hasattr HTMLParser/unescape. contextvars — hasattr
# ContextVar/Context/Token/copy_context) are covered in the matching
# pass fixture `test_subprocess_selectors_zipfile_value_ops`.
import zipfile
import tarfile
from html import entities as html_entities
import contextvars


_ledger: list[int] = []

# 1) hasattr(zipfile, 'ZipInfo') — ZipInfo metadata record
#    (mamba: returns False)
assert hasattr(zipfile, "ZipInfo") == True; _ledger.append(1)

# 2) hasattr(zipfile, 'BadZipFile') — BadZipFile exception
#    (mamba: returns False)
assert hasattr(zipfile, "BadZipFile") == True; _ledger.append(1)

# 3) hasattr(zipfile, 'ZIP_BZIP2') — ZIP_BZIP2 compression-method constant
#    (mamba: returns False)
assert hasattr(zipfile, "ZIP_BZIP2") == True; _ledger.append(1)

# 4) hasattr(tarfile, 'TarFile') — TarFile class
#    (mamba: returns False)
assert hasattr(tarfile, "TarFile") == True; _ledger.append(1)

# 5) hasattr(tarfile, 'TarInfo') — TarInfo metadata record
#    (mamba: returns False)
assert hasattr(tarfile, "TarInfo") == True; _ledger.append(1)

# 6) hasattr(tarfile, 'TarError') — TarError exception
#    (mamba: returns False)
assert hasattr(tarfile, "TarError") == True; _ledger.append(1)

# 7) hasattr(tarfile, 'USTAR_FORMAT') — USTAR_FORMAT constant
#    (mamba: returns False)
assert hasattr(tarfile, "USTAR_FORMAT") == True; _ledger.append(1)

# 8) hasattr(html.entities, 'name2codepoint') — name2codepoint dict
#    (mamba: returns False)
assert hasattr(html_entities, "name2codepoint") == True; _ledger.append(1)

# 9) hasattr(html.entities, 'codepoint2name') — codepoint2name dict
#    (mamba: returns False)
assert hasattr(html_entities, "codepoint2name") == True; _ledger.append(1)

# 10) type(contextvars.ContextVar('x')).__name__ == 'ContextVar' — ContextVar instance
#     (mamba: returns 'str' — constructor degrades to the name string)
assert type(contextvars.ContextVar("x")).__name__ == "ContextVar"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_zipfile_tarfile_html_contextvars_silent {sum(_ledger)} asserts")
