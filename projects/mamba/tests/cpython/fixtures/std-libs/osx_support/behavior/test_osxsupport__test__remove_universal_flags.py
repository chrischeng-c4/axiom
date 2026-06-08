# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "osx_support"
# dimension = "behavior"
# case = "test_osxsupport__test__remove_universal_flags"
# subject = "cpython.test__osx_support.Test_OSXSupport.test__remove_universal_flags"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test__osx_support.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Test_OSXSupport.test__remove_universal_flags strips arch and sdk flags."""
import _osx_support

config_vars = {
    "CFLAGS": "-fno-strict-aliasing  -g -O3 -arch ppc -arch i386  ",
    "LDFLAGS": "-arch ppc -arch i386   -g",
    "CPPFLAGS": "-I. -isysroot /Developer/SDKs/MacOSX10.4u.sdk",
    "BLDSHARED": "gcc-4.0 -bundle  -arch ppc -arch i386 -g",
    "LDSHARED": (
        "gcc-4.0 -bundle  -arch ppc -arch i386 "
        "-isysroot /Developer/SDKs/MacOSX10.4u.sdk -g"
    ),
}
expected_vars = {
    "CFLAGS": "-fno-strict-aliasing  -g -O3    ",
    "LDFLAGS": "-arch ppc -arch i386   -g",
    "CPPFLAGS": "-I. -isysroot /Developer/SDKs/MacOSX10.4u.sdk",
    "BLDSHARED": "gcc-4.0 -bundle    -g",
    "LDSHARED": "gcc-4.0 -bundle      -g",
}
for key, value in config_vars.items():
    if value != expected_vars[key]:
        expected_vars[f"_OSX_SUPPORT_INITIAL_{key}"] = value

actual = _osx_support._remove_universal_flags(config_vars)
assert actual == expected_vars, actual

print("Test_OSXSupport::test__remove_universal_flags: ok")
