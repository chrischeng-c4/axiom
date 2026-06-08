# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_string_fmt_socket_ssl_urllib_sys_time_value_ops"
# subject = "cpython321.test_string_fmt_socket_ssl_urllib_sys_time_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_string_fmt_socket_ssl_urllib_sys_time_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_string_fmt_socket_ssl_urllib_sys_time_value_ops: execute CPython 3.12 seed test_string_fmt_socket_ssl_urllib_sys_time_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 245 pass conformance — str.format positional/kwarg/pad/right/center/
# zero/hex/float/e specifiers + str.format_map + format() builtin + f-string
# basic/format/float / socket basic surface (socket/AF_INET/AF_INET6/
# SOCK_STREAM/SOCK_DGRAM/AF_UNIX/gethostname/gethostbyname/getaddrinfo) / ssl
# class surface (SSLContext/create_default_context/CERT_NONE/CERT_REQUIRED/
# PROTOCOL_TLS/SSLError/Purpose) / urllib.request surface (urlopen/Request/
# build_opener/install_opener/HTTPHandler) / urllib.error surface (URLError/
# HTTPError/ContentTooShortError) / os.environ hasattr + HOME type / sys
# surface (flags/version_info/platform/maxsize/executable/modules/argv/
# stdout/stderr/stdin/path) + maxsize int + platform str types / time
# surface (monotonic/perf_counter/process_time/time/sleep/time_ns/
# monotonic_ns) + monotonic/time positive value ops that match between
# CPython 3.12 and mamba.
import socket
import ssl
import urllib.request as ureq
import urllib.error as uerr
import os
import sys
import time


_ledger: list[int] = []

# 1) str.format positional + kwarg
assert "{0} {1}".format("hello", "world") == "hello world"; _ledger.append(1)
assert "{name}".format(name="x") == "x"; _ledger.append(1)

# 2) str.format alignment + padding specifiers
assert "{:<10}|".format("x") == "x         |"; _ledger.append(1)
assert "{:>10}|".format("x") == "         x|"; _ledger.append(1)
assert "{:^10}|".format("x") == "    x     |"; _ledger.append(1)
assert "{:04d}".format(42) == "0042"; _ledger.append(1)

# 3) str.format hex / float / e specifiers
assert "{:x}".format(255) == "ff"; _ledger.append(1)
assert "{:.3f}".format(3.14159) == "3.142"; _ledger.append(1)
assert "{:.2e}".format(12345.678) == "1.23e+04"; _ledger.append(1)

# 4) str.format_map + format() builtin
assert "{name}".format_map({"name": "x"}) == "x"; _ledger.append(1)
assert format(255, "x") == "ff"; _ledger.append(1)
assert format(3.14159, ".3f") == "3.142"; _ledger.append(1)

# 5) f-string basic + format + float
assert f"hello {42}" == "hello 42"; _ledger.append(1)
assert f"{255:x}" == "ff"; _ledger.append(1)
assert f"{3.14159:.3f}" == "3.142"; _ledger.append(1)

# 6) socket basic surface — hasattr conform
assert hasattr(socket, "socket") == True; _ledger.append(1)
assert hasattr(socket, "AF_INET") == True; _ledger.append(1)
assert hasattr(socket, "AF_INET6") == True; _ledger.append(1)
assert hasattr(socket, "SOCK_STREAM") == True; _ledger.append(1)
assert hasattr(socket, "SOCK_DGRAM") == True; _ledger.append(1)
assert hasattr(socket, "AF_UNIX") == True; _ledger.append(1)
assert hasattr(socket, "gethostname") == True; _ledger.append(1)
assert hasattr(socket, "gethostbyname") == True; _ledger.append(1)
assert hasattr(socket, "getaddrinfo") == True; _ledger.append(1)

# 7) ssl class surface — hasattr conform
assert hasattr(ssl, "SSLContext") == True; _ledger.append(1)
assert hasattr(ssl, "create_default_context") == True; _ledger.append(1)
assert hasattr(ssl, "CERT_NONE") == True; _ledger.append(1)
assert hasattr(ssl, "CERT_REQUIRED") == True; _ledger.append(1)
assert hasattr(ssl, "PROTOCOL_TLS") == True; _ledger.append(1)
assert hasattr(ssl, "SSLError") == True; _ledger.append(1)
assert hasattr(ssl, "Purpose") == True; _ledger.append(1)

# 8) urllib.request surface — hasattr conform
assert hasattr(ureq, "urlopen") == True; _ledger.append(1)
assert hasattr(ureq, "Request") == True; _ledger.append(1)
assert hasattr(ureq, "build_opener") == True; _ledger.append(1)
assert hasattr(ureq, "install_opener") == True; _ledger.append(1)
assert hasattr(ureq, "HTTPHandler") == True; _ledger.append(1)

# 9) urllib.error surface — hasattr conform
assert hasattr(uerr, "URLError") == True; _ledger.append(1)
assert hasattr(uerr, "HTTPError") == True; _ledger.append(1)
assert hasattr(uerr, "ContentTooShortError") == True; _ledger.append(1)

# 10) os.environ basic
assert hasattr(os, "environ") == True; _ledger.append(1)
assert type(os.environ.get("HOME", "")).__name__ == "str"; _ledger.append(1)

# 11) sys surface — hasattr conform
assert hasattr(sys, "flags") == True; _ledger.append(1)
assert hasattr(sys, "version_info") == True; _ledger.append(1)
assert hasattr(sys, "platform") == True; _ledger.append(1)
assert hasattr(sys, "maxsize") == True; _ledger.append(1)
assert hasattr(sys, "executable") == True; _ledger.append(1)
assert hasattr(sys, "modules") == True; _ledger.append(1)
assert hasattr(sys, "argv") == True; _ledger.append(1)
assert hasattr(sys, "stdout") == True; _ledger.append(1)
assert hasattr(sys, "stderr") == True; _ledger.append(1)
assert hasattr(sys, "stdin") == True; _ledger.append(1)
assert hasattr(sys, "path") == True; _ledger.append(1)
assert type(sys.maxsize).__name__ == "int"; _ledger.append(1)
assert type(sys.platform).__name__ == "str"; _ledger.append(1)

# 12) time surface — hasattr conform
assert hasattr(time, "monotonic") == True; _ledger.append(1)
assert hasattr(time, "perf_counter") == True; _ledger.append(1)
assert hasattr(time, "process_time") == True; _ledger.append(1)
assert hasattr(time, "time") == True; _ledger.append(1)
assert hasattr(time, "sleep") == True; _ledger.append(1)
assert hasattr(time, "time_ns") == True; _ledger.append(1)
assert hasattr(time, "monotonic_ns") == True; _ledger.append(1)

# 13) time monotonic / time positive value ops
assert time.monotonic() > 0; _ledger.append(1)
assert time.time() > 0; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_string_fmt_socket_ssl_urllib_sys_time_value_ops {sum(_ledger)} asserts")
