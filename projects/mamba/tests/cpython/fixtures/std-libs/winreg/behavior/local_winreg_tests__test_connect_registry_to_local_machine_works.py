# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winreg"
# dimension = "behavior"
# case = "local_winreg_tests__test_connect_registry_to_local_machine_works"
# subject = "cpython.test_winreg.LocalWinregTests.test_connect_registry_to_local_machine_works"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_winreg.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_winreg.py::LocalWinregTests::test_connect_registry_to_local_machine_works
"""Auto-ported test: LocalWinregTests::test_connect_registry_to_local_machine_works."""


try:
    import winreg
except ImportError:
    print("LocalWinregTests::test_connect_registry_to_local_machine_works: skipped, winreg unavailable")
    raise SystemExit(0)


hkey = winreg.ConnectRegistry(None, winreg.HKEY_LOCAL_MACHINE)
try:
    assert hkey.handle != 0, hkey.handle
finally:
    hkey.Close()

assert hkey.handle == 0, hkey.handle
print("LocalWinregTests::test_connect_registry_to_local_machine_works: ok")
