# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_unittest_doctest_zoneinfo_value_ops"
# subject = "cpython321.test_unittest_doctest_zoneinfo_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_unittest_doctest_zoneinfo_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_unittest_doctest_zoneinfo_value_ops: execute CPython 3.12 seed test_unittest_doctest_zoneinfo_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 316 pass conformance — unittest module (hasattr TestCase/main
# /skip/skipIf/skipUnless/expectedFailure) + doctest module (hasattr
# DocTestParser/DocTestFinder/DocTestRunner/DocTestSuite/testfile/
# testmod/Example/DocTest/OutputChecker/ELLIPSIS/SKIP/NORMALIZE_WHITE
# SPACE/REPORT_ONLY_FIRST_FAILURE/DONT_ACCEPT_TRUE_FOR_1/FAIL_FAST/
# debug/debug_script/debug_src/master) + zoneinfo module (hasattr Zone
# Info/ZoneInfoNotFoundError/InvalidTZPathWarning/TZPATH/available_
# timezones/reset_tzpath) + signal module (hasattr SIGINT/SIGTERM/
# SIGKILL/SIGABRT/SIGSEGV/SIGHUP/SIGUSR1/SIGUSR2/SIGPIPE/SIGALRM/
# SIGCHLD/signal/getsignal/SIG_DFL/SIG_IGN + SIGINT==2 + SIGTERM==15)
# + threading module (hasattr Thread/Lock/RLock/Condition/Event/
# Semaphore/Barrier/Timer/local/current_thread/main_thread/active_count
# /get_ident/TIMEOUT_MAX) + ipaddress module (hasattr IPv4Address/
# IPv6Address/IPv4Network/IPv6Network/ip_address/ip_network/Address
# ValueError/NetmaskValueError/v4_int_to_packed/v6_int_to_packed) +
# uuid module (hasattr UUID/uuid1/uuid3/uuid4/uuid5/NAMESPACE_DNS/
# NAMESPACE_URL/NAMESPACE_OID/NAMESPACE_X500/RFC_4122/SafeUUID/getnode).
# All asserts match between CPython 3.12 and mamba.
import unittest
import doctest
import zoneinfo
import signal
import threading
import ipaddress
import uuid


_ledger: list[int] = []

# 1) unittest — hasattr (conformant subset)
assert hasattr(unittest, "TestCase") == True; _ledger.append(1)
assert hasattr(unittest, "main") == True; _ledger.append(1)
assert hasattr(unittest, "skip") == True; _ledger.append(1)
assert hasattr(unittest, "skipIf") == True; _ledger.append(1)
assert hasattr(unittest, "skipUnless") == True; _ledger.append(1)
assert hasattr(unittest, "expectedFailure") == True; _ledger.append(1)

# 2) doctest — hasattr core surface
assert hasattr(doctest, "DocTestParser") == True; _ledger.append(1)
assert hasattr(doctest, "DocTestFinder") == True; _ledger.append(1)
assert hasattr(doctest, "DocTestRunner") == True; _ledger.append(1)
assert hasattr(doctest, "DocTestSuite") == True; _ledger.append(1)
assert hasattr(doctest, "testfile") == True; _ledger.append(1)
assert hasattr(doctest, "testmod") == True; _ledger.append(1)
assert hasattr(doctest, "Example") == True; _ledger.append(1)
assert hasattr(doctest, "DocTest") == True; _ledger.append(1)
assert hasattr(doctest, "OutputChecker") == True; _ledger.append(1)
assert hasattr(doctest, "ELLIPSIS") == True; _ledger.append(1)
assert hasattr(doctest, "SKIP") == True; _ledger.append(1)
assert hasattr(doctest, "NORMALIZE_WHITESPACE") == True; _ledger.append(1)
assert hasattr(doctest, "REPORT_ONLY_FIRST_FAILURE") == True; _ledger.append(1)
assert hasattr(doctest, "DONT_ACCEPT_TRUE_FOR_1") == True; _ledger.append(1)
assert hasattr(doctest, "FAIL_FAST") == True; _ledger.append(1)
assert hasattr(doctest, "debug") == True; _ledger.append(1)
assert hasattr(doctest, "debug_script") == True; _ledger.append(1)
assert hasattr(doctest, "debug_src") == True; _ledger.append(1)
assert hasattr(doctest, "master") == True; _ledger.append(1)

# 3) zoneinfo — hasattr
assert hasattr(zoneinfo, "ZoneInfo") == True; _ledger.append(1)
assert hasattr(zoneinfo, "ZoneInfoNotFoundError") == True; _ledger.append(1)
assert hasattr(zoneinfo, "InvalidTZPathWarning") == True; _ledger.append(1)
assert hasattr(zoneinfo, "TZPATH") == True; _ledger.append(1)
assert hasattr(zoneinfo, "available_timezones") == True; _ledger.append(1)
assert hasattr(zoneinfo, "reset_tzpath") == True; _ledger.append(1)

# 4) signal — hasattr (conformant subset) + int contracts
assert hasattr(signal, "SIGINT") == True; _ledger.append(1)
assert hasattr(signal, "SIGTERM") == True; _ledger.append(1)
assert hasattr(signal, "SIGKILL") == True; _ledger.append(1)
assert hasattr(signal, "SIGABRT") == True; _ledger.append(1)
assert hasattr(signal, "SIGSEGV") == True; _ledger.append(1)
assert hasattr(signal, "SIGHUP") == True; _ledger.append(1)
assert hasattr(signal, "SIGUSR1") == True; _ledger.append(1)
assert hasattr(signal, "SIGUSR2") == True; _ledger.append(1)
assert hasattr(signal, "SIGPIPE") == True; _ledger.append(1)
assert hasattr(signal, "SIGALRM") == True; _ledger.append(1)
assert hasattr(signal, "SIGCHLD") == True; _ledger.append(1)
assert hasattr(signal, "signal") == True; _ledger.append(1)
assert hasattr(signal, "getsignal") == True; _ledger.append(1)
assert hasattr(signal, "SIG_DFL") == True; _ledger.append(1)
assert hasattr(signal, "SIG_IGN") == True; _ledger.append(1)
assert signal.SIGINT == 2; _ledger.append(1)
assert signal.SIGTERM == 15; _ledger.append(1)

# 5) threading — hasattr core surface
assert hasattr(threading, "Thread") == True; _ledger.append(1)
assert hasattr(threading, "Lock") == True; _ledger.append(1)
assert hasattr(threading, "RLock") == True; _ledger.append(1)
assert hasattr(threading, "Condition") == True; _ledger.append(1)
assert hasattr(threading, "Event") == True; _ledger.append(1)
assert hasattr(threading, "Semaphore") == True; _ledger.append(1)
assert hasattr(threading, "Barrier") == True; _ledger.append(1)
assert hasattr(threading, "Timer") == True; _ledger.append(1)
assert hasattr(threading, "local") == True; _ledger.append(1)
assert hasattr(threading, "current_thread") == True; _ledger.append(1)
assert hasattr(threading, "main_thread") == True; _ledger.append(1)
assert hasattr(threading, "active_count") == True; _ledger.append(1)
assert hasattr(threading, "get_ident") == True; _ledger.append(1)
assert hasattr(threading, "TIMEOUT_MAX") == True; _ledger.append(1)

# 6) ipaddress — hasattr
assert hasattr(ipaddress, "IPv4Address") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv6Address") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv4Network") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv6Network") == True; _ledger.append(1)
assert hasattr(ipaddress, "ip_address") == True; _ledger.append(1)
assert hasattr(ipaddress, "ip_network") == True; _ledger.append(1)
assert hasattr(ipaddress, "AddressValueError") == True; _ledger.append(1)
assert hasattr(ipaddress, "NetmaskValueError") == True; _ledger.append(1)
assert hasattr(ipaddress, "v4_int_to_packed") == True; _ledger.append(1)
assert hasattr(ipaddress, "v6_int_to_packed") == True; _ledger.append(1)

# 7) uuid — hasattr
assert hasattr(uuid, "UUID") == True; _ledger.append(1)
assert hasattr(uuid, "uuid1") == True; _ledger.append(1)
assert hasattr(uuid, "uuid3") == True; _ledger.append(1)
assert hasattr(uuid, "uuid4") == True; _ledger.append(1)
assert hasattr(uuid, "uuid5") == True; _ledger.append(1)
assert hasattr(uuid, "NAMESPACE_DNS") == True; _ledger.append(1)
assert hasattr(uuid, "NAMESPACE_URL") == True; _ledger.append(1)
assert hasattr(uuid, "NAMESPACE_OID") == True; _ledger.append(1)
assert hasattr(uuid, "NAMESPACE_X500") == True; _ledger.append(1)
assert hasattr(uuid, "RFC_4122") == True; _ledger.append(1)
assert hasattr(uuid, "SafeUUID") == True; _ledger.append(1)
assert hasattr(uuid, "getnode") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_unittest_doctest_zoneinfo_value_ops {sum(_ledger)} asserts")
