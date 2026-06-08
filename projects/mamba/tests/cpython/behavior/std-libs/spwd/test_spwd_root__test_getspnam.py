# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "spwd"
# dimension = "behavior"
# case = "test_spwd_root__test_getspnam"
# subject = "cpython.test_spwd.TestSpwdRoot.test_getspnam"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_spwd.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_spwd.py::TestSpwdRoot::test_getspnam
"""Auto-ported test: TestSpwdRoot::test_getspnam."""


import os
import warnings


try:
    with warnings.catch_warnings():
        warnings.simplefilter("ignore", DeprecationWarning)
        import spwd
except ImportError:
    print("TestSpwdRoot::test_getspnam: skipped spwd unavailable")
else:
    if not (hasattr(os, "geteuid") and os.geteuid() == 0):
        print("TestSpwdRoot::test_getspnam: skipped root privileges required")
    else:
        entries = spwd.getspall()
        if not entries:
            print("TestSpwdRoot::test_getspnam: skipped empty shadow password database")
        else:
            random_name = entries[0].sp_namp
            entry = spwd.getspnam(random_name)
            assert isinstance(entry, spwd.struct_spwd)
            assert entry.sp_namp == random_name
            assert entry.sp_namp == entry[0]
            assert entry.sp_namp == entry.sp_nam
            assert isinstance(entry.sp_pwdp, str)
            assert entry.sp_pwdp == entry[1]
            assert entry.sp_pwdp == entry.sp_pwd
            assert isinstance(entry.sp_lstchg, int)
            assert entry.sp_lstchg == entry[2]
            assert isinstance(entry.sp_min, int)
            assert entry.sp_min == entry[3]
            assert isinstance(entry.sp_max, int)
            assert entry.sp_max == entry[4]
            assert isinstance(entry.sp_warn, int)
            assert entry.sp_warn == entry[5]
            assert isinstance(entry.sp_inact, int)
            assert entry.sp_inact == entry[6]
            assert isinstance(entry.sp_expire, int)
            assert entry.sp_expire == entry[7]
            assert isinstance(entry.sp_flag, int)
            assert entry.sp_flag == entry[8]
            try:
                spwd.getspnam("invalid user name")
            except KeyError as exc:
                assert str(exc) == "'getspnam(): name not found'"
            else:
                raise AssertionError("expected KeyError for invalid user name")
            for args in [(), (0,), (random_name, 0)]:
                try:
                    spwd.getspnam(*args)
                except TypeError:
                    pass
                else:
                    raise AssertionError(f"expected TypeError for args={args!r}")
            try:
                bytes_name = os.fsencode(random_name)
            except UnicodeEncodeError:
                pass
            else:
                try:
                    spwd.getspnam(bytes_name)
                except TypeError:
                    pass
                else:
                    raise AssertionError("expected TypeError for bytes name")
            print("TestSpwdRoot::test_getspnam: ok")
