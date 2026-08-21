# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_imaplib_ftplib_telnetlib_http_cookies_silent"
# subject = "cpython321.lang_imaplib_ftplib_telnetlib_http_cookies_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_imaplib_ftplib_telnetlib_http_cookies_silent.py"
# status = "filled"
# ///
"""cpython321.lang_imaplib_ftplib_telnetlib_http_cookies_silent: execute CPython 3.12 seed lang_imaplib_ftplib_telnetlib_http_cookies_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the `imaplib` /
# `ftplib` / `telnetlib` / `http.cookies` / `http.cookiejar`
# / `urllib.error` / `urllib.response` / `urllib.robotparser`
# eight-pack pinned to atomic 223: `imaplib` (the documented
# `hasattr(imaplib, "Commands") == True` extended hasattr
# surface), `ftplib` (the documented `hasattr(ftplib,
# "parse150") / "parse227" / "parse229" / "parse257" /
# "print_line" / "ftpcp" == True` extended hasattr surface),
# `telnetlib` (the documented `hasattr(telnetlib, "ECHO") /
# "TTYPE" / "EOR" / "BINARY" / "SGA" == True` extended
# hasattr surface), `http.cookies` (the documented
# `hasattr(http.cookies, "CookieError") / "BaseCookie" /
# "SimpleCookie" / "Morsel" == True` extended hasattr
# surface), `http.cookiejar` (the documented
# `hasattr(http.cookiejar, "CookieJar") / "FileCookieJar" /
# "MozillaCookieJar" / "LWPCookieJar" / "Cookie" /
# "DefaultCookiePolicy" / "CookiePolicy" / "LoadError" ==
# True` extended hasattr surface), `urllib.error` (the
# documented `hasattr(urllib.error, "URLError") / "HTTPError"
# / "ContentTooShortError" == True` extended hasattr surface),
# `urllib.response` (the documented `hasattr(urllib.response,
# "addinfourl") / "addbase" / "addclosehook" / "addinfo" ==
# True` extended hasattr surface), and `urllib.robotparser`
# (the documented `hasattr(urllib.robotparser, "RobotFileParser")
# == True` extended hasattr surface + the documented
# `type(RobotFileParser()).__name__ == "RobotFileParser"`
# constructor value contract via from-import).
#
# Behavioral edges that CONFORM on mamba (smtplib full
# hasattr surface, poplib full hasattr surface, imaplib
# IMAP4 / IMAP4_SSL / IMAP4_PORT / IMAP4_SSL_PORT /
# Internaldate2tuple / Int2AP / ParseFlags /
# Time2Internaldate hasattr, ftplib FTP / FTP_TLS / FTP_PORT
# / MSG_OOB / MAXLINE / error_reply / error_temp /
# error_perm / error_proto / all_errors hasattr, telnetlib
# Telnet / TELNET_PORT / IAC / DONT / DO / WONT / WILL / SE
# / NOP / DM / BRK / IP / AO / AYT / EC / EL / GA / SB
# hasattr, nntplib full hasattr surface) are covered in the
# matching pass fixture
# `test_smtplib_poplib_imaplib_ftplib_telnetlib_nntplib_value_ops`.
import warnings as _warnings_mod_real

_warnings_mod_real.filterwarnings("ignore", category=DeprecationWarning)

from typing import Any
import imaplib as _imaplib_mod
import ftplib as _ftplib_mod
import telnetlib as _telnetlib_mod
import http.cookies as _cookies_mod
import http.cookiejar as _cookiejar_mod
import urllib.error as _urlerror_mod
import urllib.response as _urlresponse_mod
import urllib.robotparser as _urlrobotparser_mod
from urllib.robotparser import RobotFileParser as _RobotFileParser

imaplib: Any = _imaplib_mod
ftplib: Any = _ftplib_mod
telnetlib: Any = _telnetlib_mod
cookies: Any = _cookies_mod
cookiejar: Any = _cookiejar_mod
urlerror: Any = _urlerror_mod
urlresponse: Any = _urlresponse_mod
urlrobotparser: Any = _urlrobotparser_mod


_ledger: list[int] = []

# 1) imaplib — extended module hasattr surface
#    (mamba: Commands False)
assert hasattr(imaplib, "Commands") == True; _ledger.append(1)

# 2) ftplib — extended module hasattr surface
#    (mamba: parse150 / parse227 / parse229 / parse257 /
#    print_line / ftpcp all False)
assert hasattr(ftplib, "parse150") == True; _ledger.append(1)
assert hasattr(ftplib, "parse227") == True; _ledger.append(1)
assert hasattr(ftplib, "parse229") == True; _ledger.append(1)
assert hasattr(ftplib, "parse257") == True; _ledger.append(1)
assert hasattr(ftplib, "print_line") == True; _ledger.append(1)
assert hasattr(ftplib, "ftpcp") == True; _ledger.append(1)

# 3) telnetlib — extended module hasattr surface
#    (mamba: ECHO / TTYPE / EOR / BINARY / SGA all False)
assert hasattr(telnetlib, "ECHO") == True; _ledger.append(1)
assert hasattr(telnetlib, "TTYPE") == True; _ledger.append(1)
assert hasattr(telnetlib, "EOR") == True; _ledger.append(1)
assert hasattr(telnetlib, "BINARY") == True; _ledger.append(1)
assert hasattr(telnetlib, "SGA") == True; _ledger.append(1)

# 4) http.cookies — extended module hasattr surface
#    (mamba: dotted access collapses, CookieError /
#    BaseCookie / SimpleCookie / Morsel all False)
assert hasattr(cookies, "CookieError") == True; _ledger.append(1)
assert hasattr(cookies, "BaseCookie") == True; _ledger.append(1)
assert hasattr(cookies, "SimpleCookie") == True; _ledger.append(1)
assert hasattr(cookies, "Morsel") == True; _ledger.append(1)

# 5) http.cookiejar — extended module hasattr surface
#    (mamba: dotted access collapses, all 8 attrs False)
assert hasattr(cookiejar, "CookieJar") == True; _ledger.append(1)
assert hasattr(cookiejar, "FileCookieJar") == True; _ledger.append(1)
assert hasattr(cookiejar, "MozillaCookieJar") == True; _ledger.append(1)
assert hasattr(cookiejar, "LWPCookieJar") == True; _ledger.append(1)
assert hasattr(cookiejar, "Cookie") == True; _ledger.append(1)
assert hasattr(cookiejar, "DefaultCookiePolicy") == True; _ledger.append(1)
assert hasattr(cookiejar, "CookiePolicy") == True; _ledger.append(1)
assert hasattr(cookiejar, "LoadError") == True; _ledger.append(1)

# 6) urllib.error — extended module hasattr surface
#    (mamba: dotted access collapses, URLError / HTTPError /
#    ContentTooShortError all False)
assert hasattr(urlerror, "URLError") == True; _ledger.append(1)
assert hasattr(urlerror, "HTTPError") == True; _ledger.append(1)
assert hasattr(urlerror, "ContentTooShortError") == True; _ledger.append(1)

# 7) urllib.response — extended module hasattr surface
#    (mamba: dotted access collapses, addinfourl / addbase /
#    addclosehook / addinfo all False)
assert hasattr(urlresponse, "addinfourl") == True; _ledger.append(1)
assert hasattr(urlresponse, "addbase") == True; _ledger.append(1)
assert hasattr(urlresponse, "addclosehook") == True; _ledger.append(1)
assert hasattr(urlresponse, "addinfo") == True; _ledger.append(1)

# 8) urllib.robotparser — extended module hasattr +
#    constructor value contract (mamba: dotted hasattr
#    returns False; from-import constructor returns dict
#    instead of RobotFileParser)
assert hasattr(urlrobotparser, "RobotFileParser") == True; _ledger.append(1)
_rp = _RobotFileParser()
assert type(_rp).__name__ == "RobotFileParser"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_imaplib_ftplib_telnetlib_http_cookies_silent {sum(_ledger)} asserts")
