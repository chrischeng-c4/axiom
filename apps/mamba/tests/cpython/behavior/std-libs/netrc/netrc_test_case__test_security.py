# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "netrc"
# dimension = "behavior"
# case = "netrc_test_case__test_security"
# subject = "cpython.test_netrc.NetrcTestCase.test_security"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_netrc.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_netrc.py::NetrcTestCase::test_security
"""Auto-ported test: NetrcTestCase::test_security (CPython 3.12 oracle)."""


import netrc
import os
from test.support import os_helper


try:
    import pwd  # noqa: F401
except ImportError:
    print("NetrcTestCase::test_security: skipped")
    raise SystemExit(0)

if os.name != "posix":
    print("NetrcTestCase::test_security: skipped")
    raise SystemExit(0)


with os_helper.temp_dir() as home:
    filename = os.path.join(home, ".netrc")
    with open(filename, "wt", encoding="utf-8") as file:
        file.write(
            """\
                machine foo.domain.com login bar password pass
                default login foo password pass
                """
        )
    with os_helper.EnvironmentVarGuard() as environ:
        environ.set("HOME", home)
        os.chmod(filename, 0o600)
        parsed = netrc.netrc()
        assert parsed.hosts["foo.domain.com"] == ("bar", "", "pass")
        os.chmod(filename, 0o622)
        try:
            netrc.netrc()
        except netrc.NetrcParseError:
            pass
        else:
            raise AssertionError("expected NetrcParseError for writable .netrc")

    with open(filename, "wt", encoding="utf-8") as file:
        file.write(
            """\
                machine foo.domain.com login anonymous password pass
                default login foo password pass
                """
        )
    with os_helper.EnvironmentVarGuard() as environ:
        environ.set("HOME", home)
        os.chmod(filename, 0o600)
        parsed = netrc.netrc()
        assert parsed.hosts["foo.domain.com"] == ("anonymous", "", "pass")
        os.chmod(filename, 0o622)
        assert parsed.hosts["foo.domain.com"] == ("anonymous", "", "pass")

print("NetrcTestCase::test_security: ok")
