# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_gzip_zipfile_tarfile_socket_typing_io_silent"
# subject = "cpython321.lang_gzip_zipfile_tarfile_socket_typing_io_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_gzip_zipfile_tarfile_socket_typing_io_silent.py"
# status = "filled"
# ///
"""cpython321.lang_gzip_zipfile_tarfile_socket_typing_io_silent: execute CPython 3.12 seed lang_gzip_zipfile_tarfile_socket_typing_io_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the
# `gzip` / `zipfile` / `tarfile` / `socket` / `typing` /
# `io` six-pack pinned to atomic 230:
# `gzip` (the documented `hasattr(gzip, "READ") / "WRITE"
# == True` mode-constant surface), `zipfile` (the documented
# extended `hasattr(zipfile, "ZipInfo") / "BadZipFile" /
# "BadZipfile" / "LargeZipFile" / "ZIP_BZIP2" / "ZIP_LZMA" /
# "Path" / "PyZipFile" == True` extended hasattr surface),
# `tarfile` (the documented extended `hasattr(tarfile,
# "TarFile") / "TarInfo" / "TarError" / "ReadError" /
# "CompressionError" / "StreamError" / "ExtractError" /
# "HeaderError" / "USTAR_FORMAT" / "GNU_FORMAT" /
# "PAX_FORMAT" / "DEFAULT_FORMAT" / "ENCODING" / "REGTYPE" /
# "LNKTYPE" / "SYMTYPE" / "DIRTYPE" / "FIFOTYPE" /
# "CONTTYPE" / "BLKTYPE" / "CHRTYPE" == True` extended
# hasattr surface), `socket` (the documented extended
# `hasattr(socket, "SOCK_RAW") / "SOL_SOCKET" / "SO_REUSEADDR"
# / "SO_REUSEPORT" / "SO_KEEPALIVE" / "IPPROTO_TCP" /
# "IPPROTO_UDP" / "IPPROTO_IP" / "IPPROTO_IPV6" /
# "gethostbyaddr" / "getnameinfo" / "getfqdn" / "getservbyname"
# / "getservbyport" / "getdefaulttimeout" / "setdefaulttimeout"
# / "has_ipv6" / "error" / "herror" / "gaierror" / "timeout"
# / "SocketType" == True` extended hasattr surface),
# `typing` (the documented extended `hasattr(typing,
# "Iterable") / "Sequence" / "Mapping" / "MutableMapping" /
# "MutableSequence" / "MutableSet" / "Hashable" / "Sized" /
# "Container" / "runtime_checkable" / "no_type_check" /
# "Annotated" / "overload" / "NewType" / "ParamSpec" /
# "TypeAlias" / "Concatenate" / "Self" == True` extended
# ABC + decorator + PEP-695 surface), and `io` (the
# documented extended `hasattr(io, "IOBase") / "RawIOBase" /
# "BufferedIOBase" / "TextIOBase" / "FileIO" / "BufferedReader"
# / "BufferedWriter" / "BufferedRandom" / "BufferedRWPair" /
# "TextIOWrapper" / "IncrementalNewlineDecoder" /
# "DEFAULT_BUFFER_SIZE" / "SEEK_SET" / "SEEK_CUR" / "SEEK_END"
# / "UnsupportedOperation" / "BlockingIOError" / "open" ==
# True` extended IO base-class + seek-constant surface).
#
# Behavioral edges that CONFORM on mamba (gzip/bz2/lzma round-
# trip + core surface, zipfile core, tarfile core, socket
# common surface + gethostname value-op, ssl full surface,
# typing core, numbers full ABC, selectors core, shelve full
# surface) are covered in the matching pass fixture
# `test_gzip_bz2_lzma_ssl_typing_numbers_value_ops`.
from typing import Any
import gzip as _gzip_mod
import zipfile as _zipfile_mod
import tarfile as _tarfile_mod
import socket as _socket_mod
import typing as _typing_mod
import io as _io_mod

gzip: Any = _gzip_mod
zipfile: Any = _zipfile_mod
tarfile: Any = _tarfile_mod
socket: Any = _socket_mod
typing: Any = _typing_mod
io: Any = _io_mod


_ledger: list[int] = []

# 1) gzip — mode-constant surface hasattr
#    (mamba: READ / WRITE both False)
assert hasattr(gzip, "READ") == True; _ledger.append(1)
assert hasattr(gzip, "WRITE") == True; _ledger.append(1)

# 2) zipfile — extended module hasattr surface
#    (mamba: ZipInfo / BadZipFile / BadZipfile / LargeZipFile /
#    ZIP_BZIP2 / ZIP_LZMA / Path / PyZipFile all False)
assert hasattr(zipfile, "ZipInfo") == True; _ledger.append(1)
assert hasattr(zipfile, "BadZipFile") == True; _ledger.append(1)
assert hasattr(zipfile, "BadZipfile") == True; _ledger.append(1)
assert hasattr(zipfile, "LargeZipFile") == True; _ledger.append(1)
assert hasattr(zipfile, "ZIP_BZIP2") == True; _ledger.append(1)
assert hasattr(zipfile, "ZIP_LZMA") == True; _ledger.append(1)
assert hasattr(zipfile, "Path") == True; _ledger.append(1)
assert hasattr(zipfile, "PyZipFile") == True; _ledger.append(1)

# 3) tarfile — extended module hasattr surface
#    (mamba: TarFile / TarInfo / TarError / ReadError /
#    CompressionError / StreamError / ExtractError /
#    HeaderError / USTAR_FORMAT / GNU_FORMAT / PAX_FORMAT /
#    DEFAULT_FORMAT / ENCODING / REGTYPE / LNKTYPE / SYMTYPE
#    / DIRTYPE / FIFOTYPE / CONTTYPE / BLKTYPE / CHRTYPE all
#    False)
assert hasattr(tarfile, "TarFile") == True; _ledger.append(1)
assert hasattr(tarfile, "TarInfo") == True; _ledger.append(1)
assert hasattr(tarfile, "TarError") == True; _ledger.append(1)
assert hasattr(tarfile, "ReadError") == True; _ledger.append(1)
assert hasattr(tarfile, "CompressionError") == True; _ledger.append(1)
assert hasattr(tarfile, "StreamError") == True; _ledger.append(1)
assert hasattr(tarfile, "ExtractError") == True; _ledger.append(1)
assert hasattr(tarfile, "HeaderError") == True; _ledger.append(1)
assert hasattr(tarfile, "USTAR_FORMAT") == True; _ledger.append(1)
assert hasattr(tarfile, "GNU_FORMAT") == True; _ledger.append(1)
assert hasattr(tarfile, "PAX_FORMAT") == True; _ledger.append(1)
assert hasattr(tarfile, "DEFAULT_FORMAT") == True; _ledger.append(1)
assert hasattr(tarfile, "ENCODING") == True; _ledger.append(1)
assert hasattr(tarfile, "REGTYPE") == True; _ledger.append(1)
assert hasattr(tarfile, "LNKTYPE") == True; _ledger.append(1)
assert hasattr(tarfile, "SYMTYPE") == True; _ledger.append(1)
assert hasattr(tarfile, "DIRTYPE") == True; _ledger.append(1)
assert hasattr(tarfile, "FIFOTYPE") == True; _ledger.append(1)
assert hasattr(tarfile, "CONTTYPE") == True; _ledger.append(1)
assert hasattr(tarfile, "BLKTYPE") == True; _ledger.append(1)
assert hasattr(tarfile, "CHRTYPE") == True; _ledger.append(1)

# 4) socket — extended module hasattr surface
#    (mamba: SOCK_RAW / SOL_SOCKET / SO_REUSEADDR /
#    SO_REUSEPORT / SO_KEEPALIVE / IPPROTO_TCP / IPPROTO_UDP
#    / IPPROTO_IP / IPPROTO_IPV6 / gethostbyaddr /
#    getnameinfo / getfqdn / getservbyname / getservbyport /
#    getdefaulttimeout / setdefaulttimeout / has_ipv6 /
#    error / herror / gaierror / timeout / SocketType all
#    False)
assert hasattr(socket, "SOCK_RAW") == True; _ledger.append(1)
assert hasattr(socket, "SOL_SOCKET") == True; _ledger.append(1)
assert hasattr(socket, "SO_REUSEADDR") == True; _ledger.append(1)
assert hasattr(socket, "SO_REUSEPORT") == True; _ledger.append(1)
assert hasattr(socket, "SO_KEEPALIVE") == True; _ledger.append(1)
assert hasattr(socket, "IPPROTO_TCP") == True; _ledger.append(1)
assert hasattr(socket, "IPPROTO_UDP") == True; _ledger.append(1)
assert hasattr(socket, "IPPROTO_IP") == True; _ledger.append(1)
assert hasattr(socket, "IPPROTO_IPV6") == True; _ledger.append(1)
assert hasattr(socket, "gethostbyaddr") == True; _ledger.append(1)
assert hasattr(socket, "getnameinfo") == True; _ledger.append(1)
assert hasattr(socket, "getfqdn") == True; _ledger.append(1)
assert hasattr(socket, "getservbyname") == True; _ledger.append(1)
assert hasattr(socket, "getservbyport") == True; _ledger.append(1)
assert hasattr(socket, "getdefaulttimeout") == True; _ledger.append(1)
assert hasattr(socket, "setdefaulttimeout") == True; _ledger.append(1)
assert hasattr(socket, "has_ipv6") == True; _ledger.append(1)
assert hasattr(socket, "error") == True; _ledger.append(1)
assert hasattr(socket, "herror") == True; _ledger.append(1)
assert hasattr(socket, "gaierror") == True; _ledger.append(1)
assert hasattr(socket, "timeout") == True; _ledger.append(1)
assert hasattr(socket, "SocketType") == True; _ledger.append(1)

# 5) typing — extended ABC + decorator + PEP-695 surface
#    (mamba: Iterable / Sequence / Mapping / MutableMapping /
#    MutableSequence / MutableSet / Hashable / Sized /
#    Container / runtime_checkable / no_type_check /
#    Annotated / overload / NewType / ParamSpec / TypeAlias
#    / Concatenate / Self all False)
assert hasattr(typing, "Iterable") == True; _ledger.append(1)
assert hasattr(typing, "Sequence") == True; _ledger.append(1)
assert hasattr(typing, "Mapping") == True; _ledger.append(1)
assert hasattr(typing, "MutableMapping") == True; _ledger.append(1)
assert hasattr(typing, "MutableSequence") == True; _ledger.append(1)
assert hasattr(typing, "MutableSet") == True; _ledger.append(1)
assert hasattr(typing, "Hashable") == True; _ledger.append(1)
assert hasattr(typing, "Sized") == True; _ledger.append(1)
assert hasattr(typing, "Container") == True; _ledger.append(1)
assert hasattr(typing, "runtime_checkable") == True; _ledger.append(1)
assert hasattr(typing, "no_type_check") == True; _ledger.append(1)
assert hasattr(typing, "Annotated") == True; _ledger.append(1)
assert hasattr(typing, "overload") == True; _ledger.append(1)
assert hasattr(typing, "NewType") == True; _ledger.append(1)
assert hasattr(typing, "ParamSpec") == True; _ledger.append(1)
assert hasattr(typing, "TypeAlias") == True; _ledger.append(1)
assert hasattr(typing, "Concatenate") == True; _ledger.append(1)
assert hasattr(typing, "Self") == True; _ledger.append(1)

# 6) io — extended IO base-class + seek-constant surface
#    (mamba: IOBase / RawIOBase / BufferedIOBase / TextIOBase
#    / FileIO / BufferedReader / BufferedWriter /
#    BufferedRandom / BufferedRWPair / TextIOWrapper /
#    IncrementalNewlineDecoder / DEFAULT_BUFFER_SIZE /
#    SEEK_SET / SEEK_CUR / SEEK_END / UnsupportedOperation /
#    BlockingIOError / open all False)
assert hasattr(io, "IOBase") == True; _ledger.append(1)
assert hasattr(io, "RawIOBase") == True; _ledger.append(1)
assert hasattr(io, "BufferedIOBase") == True; _ledger.append(1)
assert hasattr(io, "TextIOBase") == True; _ledger.append(1)
assert hasattr(io, "FileIO") == True; _ledger.append(1)
assert hasattr(io, "BufferedReader") == True; _ledger.append(1)
assert hasattr(io, "BufferedWriter") == True; _ledger.append(1)
assert hasattr(io, "BufferedRandom") == True; _ledger.append(1)
assert hasattr(io, "BufferedRWPair") == True; _ledger.append(1)
assert hasattr(io, "TextIOWrapper") == True; _ledger.append(1)
assert hasattr(io, "IncrementalNewlineDecoder") == True; _ledger.append(1)
assert hasattr(io, "DEFAULT_BUFFER_SIZE") == True; _ledger.append(1)
assert hasattr(io, "SEEK_SET") == True; _ledger.append(1)
assert hasattr(io, "SEEK_CUR") == True; _ledger.append(1)
assert hasattr(io, "SEEK_END") == True; _ledger.append(1)
assert hasattr(io, "UnsupportedOperation") == True; _ledger.append(1)
assert hasattr(io, "BlockingIOError") == True; _ledger.append(1)
assert hasattr(io, "open") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_gzip_zipfile_tarfile_socket_typing_io_silent {sum(_ledger)} asserts")
