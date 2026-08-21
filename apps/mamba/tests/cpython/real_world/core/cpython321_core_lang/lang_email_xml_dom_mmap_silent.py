# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_email_xml_dom_mmap_silent"
# subject = "cpython321.lang_email_xml_dom_mmap_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_email_xml_dom_mmap_silent.py"
# status = "filled"
# ///
"""cpython321.lang_email_xml_dom_mmap_silent: execute CPython 3.12 seed lang_email_xml_dom_mmap_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(email.message, 'Message')`
# (the documented "email.message exposes the Message class" — mamba
# returns False — module body resolves to None), `hasattr(email.
# message, 'EmailMessage')` (the documented "email.message exposes
# the EmailMessage class" — mamba returns False — module body
# resolves to None), `hasattr(email.utils, 'parseaddr')` (the
# documented "email.utils exposes the parseaddr helper" — mamba
# returns False — module body resolves to None), `hasattr(email.utils,
# 'formataddr')` (the documented "email.utils exposes the formataddr
# helper" — mamba returns False — module body resolves to None), `
# hasattr(email.parser, 'Parser')` (the documented "email.parser
# exposes the Parser class" — mamba returns False — module body
# resolves to None), `hasattr(xml.dom.minidom, 'parseString')` (the
# documented "xml.dom.minidom exposes the parseString helper" — mamba
# returns False — module body resolves to None), `hasattr(xml.dom.
# minidom, 'Document')` (the documented "xml.dom.minidom exposes the
# Document class" — mamba returns False — module body resolves to
# None), `hasattr(mmap, 'mmap')` (the documented "mmap exposes the
# mmap class" — mamba returns False), `hasattr(mmap, 'PAGESIZE')`
# (the documented "mmap exposes the PAGESIZE constant" — mamba
# returns False), and `mmap.ACCESS_READ == 1` (the documented "mmap.
# ACCESS_READ is the integer 1" — mamba returns None — constant
# unresolved).
# Ten-pack pinned to atomic 310.
#
# Behavioral edges that CONFORM on mamba (http.client — hasattr HTTP
# Connection/HTTPSConnection/HTTPResponse/HTTPException/OK/NOT_FOUND/
# INTERNAL_SERVER_ERROR/responses/HTTPS_PORT/HTTP_PORT + OK == 200 +
# NOT_FOUND == 404 + INTERNAL_SERVER_ERROR == 500 + HTTP_PORT == 80 +
# HTTPS_PORT == 443. http.cookies — hasattr SimpleCookie/BaseCookie/
# Morsel/CookieError + type(SimpleCookie()) == 'SimpleCookie'. urllib
# .error — hasattr URLError/HTTPError/ContentTooShortError) are
# covered in the matching pass fixture
# `test_http_client_cookies_urllib_error_value_ops`.
from email import message as email_message
from email import utils as email_utils
from email import parser as email_parser
from xml.dom import minidom
import mmap


_ledger: list[int] = []

# 1) hasattr(email.message, 'Message') — Message class
#    (mamba: returns False — module body resolves to None)
assert hasattr(email_message, "Message") == True; _ledger.append(1)

# 2) hasattr(email.message, 'EmailMessage') — EmailMessage class
#    (mamba: returns False — module body resolves to None)
assert hasattr(email_message, "EmailMessage") == True; _ledger.append(1)

# 3) hasattr(email.utils, 'parseaddr') — parseaddr helper
#    (mamba: returns False — module body resolves to None)
assert hasattr(email_utils, "parseaddr") == True; _ledger.append(1)

# 4) hasattr(email.utils, 'formataddr') — formataddr helper
#    (mamba: returns False — module body resolves to None)
assert hasattr(email_utils, "formataddr") == True; _ledger.append(1)

# 5) hasattr(email.parser, 'Parser') — Parser class
#    (mamba: returns False — module body resolves to None)
assert hasattr(email_parser, "Parser") == True; _ledger.append(1)

# 6) hasattr(xml.dom.minidom, 'parseString') — parseString helper
#    (mamba: returns False — module body resolves to None)
assert hasattr(minidom, "parseString") == True; _ledger.append(1)

# 7) hasattr(xml.dom.minidom, 'Document') — Document class
#    (mamba: returns False — module body resolves to None)
assert hasattr(minidom, "Document") == True; _ledger.append(1)

# 8) hasattr(mmap, 'mmap') — mmap class
#    (mamba: returns False)
assert hasattr(mmap, "mmap") == True; _ledger.append(1)

# 9) hasattr(mmap, 'PAGESIZE') — PAGESIZE constant
#    (mamba: returns False)
assert hasattr(mmap, "PAGESIZE") == True; _ledger.append(1)

# 10) mmap.ACCESS_READ == 1 — read-access constant
#     (mamba: returns None — constant unresolved)
assert mmap.ACCESS_READ == 1; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_email_xml_dom_mmap_silent {sum(_ledger)} asserts")
