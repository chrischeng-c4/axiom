# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wave"
# dimension = "behavior"
# case = "misc_test_case__test_all"
# subject = "cpython.test_wave.MiscTestCase.test__all__"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_wave.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_wave.py::MiscTestCase::test__all__
"""Auto-ported test: MiscTestCase::test__all__ (CPython 3.12 oracle)."""


import unittest
import wave
from test import support


not_exported = {
    "WAVE_FORMAT_PCM",
    "WAVE_FORMAT_EXTENSIBLE",
    "KSDATAFORMAT_SUBTYPE_PCM",
}
support.check__all__(unittest.TestCase(), wave, not_exported=not_exported)

print("MiscTestCase::test__all__: ok")
