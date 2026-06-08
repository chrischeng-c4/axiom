# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "c_locale_coercion"
# dimension = "behavior"
# case = "locale_coercion_tests__test_lc_all_set_to_c"
# subject = "cpython.test_c_locale_coercion.LocaleCoercionTests.test_LC_ALL_set_to_C"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_c_locale_coercion.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_c_locale_coercion.py::LocaleCoercionTests::test_LC_ALL_set_to_C
"""Auto-ported test: LocaleCoercionTests::test_LC_ALL_set_to_C (CPython 3.12 oracle)."""


import locale
import os
import subprocess
import sys
import sysconfig
import unittest
from collections import namedtuple
from test import support
from test.support.script_helper import run_python_until_end


EXPECTED_C_LOCALE_EQUIVALENTS = ['C', 'invalid.ascii']

EXPECTED_C_LOCALE_STREAM_ENCODING = 'ascii'

EXPECTED_C_LOCALE_FS_ENCODING = 'ascii'

EXPECT_COERCION_IN_DEFAULT_LOCALE = True

TARGET_LOCALES = ['C.UTF-8', 'C.utf8', 'UTF-8']

if sys.platform.startswith('linux'):
    if support.is_android:
        EXPECTED_C_LOCALE_STREAM_ENCODING = 'utf-8'
        EXPECTED_C_LOCALE_FS_ENCODING = 'utf-8'
    else:
        EXPECTED_C_LOCALE_EQUIVALENTS.append('POSIX')
elif sys.platform.startswith('aix'):
    EXPECTED_C_LOCALE_STREAM_ENCODING = 'iso8859-1'
    EXPECTED_C_LOCALE_FS_ENCODING = 'iso8859-1'
elif sys.platform == 'darwin':
    EXPECTED_C_LOCALE_FS_ENCODING = 'utf-8'
elif sys.platform == 'cygwin':
    EXPECT_COERCION_IN_DEFAULT_LOCALE = False
elif sys.platform == 'vxworks':
    EXPECTED_C_LOCALE_STREAM_ENCODING = 'utf-8'
    EXPECTED_C_LOCALE_FS_ENCODING = 'utf-8'

_C_UTF8_LOCALES = ('C.UTF-8', 'C.utf8', 'UTF-8')

_check_nl_langinfo_CODESET = bool(sys.platform not in ('darwin', 'linux') and hasattr(locale, 'nl_langinfo') and hasattr(locale, 'CODESET'))

def _set_locale_in_subprocess(locale_name):
    cmd_fmt = "import locale; print(locale.setlocale(locale.LC_CTYPE, '{}'))"
    if _check_nl_langinfo_CODESET:
        cmd_fmt += '; import sys; sys.exit(not locale.nl_langinfo(locale.CODESET))'
    cmd = cmd_fmt.format(locale_name)
    result, py_cmd = run_python_until_end('-c', cmd, PYTHONCOERCECLOCALE='')
    return result.rc == 0

_fields = 'fsencoding stdin_info stdout_info stderr_info lang lc_ctype lc_all'

_EncodingDetails = namedtuple('EncodingDetails', _fields)

class EncodingDetails(_EncodingDetails):
    CHILD_PROCESS_SCRIPT = ';'.join(['import sys, os', 'print(sys.getfilesystemencoding())', "print(sys.stdin.encoding + ':' + sys.stdin.errors)", "print(sys.stdout.encoding + ':' + sys.stdout.errors)", "print(sys.stderr.encoding + ':' + sys.stderr.errors)", "print(os.environ.get('LANG', 'not set'))", "print(os.environ.get('LC_CTYPE', 'not set'))", "print(os.environ.get('LC_ALL', 'not set'))"])

    @classmethod
    def get_expected_details(cls, coercion_expected, fs_encoding, stream_encoding, stream_errors, env_vars):
        """Returns expected child process details for a given encoding"""
        _stream = stream_encoding + ':{}'
        if stream_errors is None:
            stream_errors = 'surrogateescape'
        stream_info = [_stream.format(stream_errors)] * 2
        stream_info.append(_stream.format('backslashreplace'))
        expected_lang = env_vars.get('LANG', 'not set')
        if coercion_expected:
            expected_lc_ctype = CLI_COERCION_TARGET
        else:
            expected_lc_ctype = env_vars.get('LC_CTYPE', 'not set')
        expected_lc_all = env_vars.get('LC_ALL', 'not set')
        env_info = (expected_lang, expected_lc_ctype, expected_lc_all)
        return dict(cls(fs_encoding, *stream_info, *env_info)._asdict())

    @classmethod
    def get_child_details(cls, env_vars):
        """Retrieves fsencoding and standard stream details from a child process

        Returns (encoding_details, stderr_lines):

        - encoding_details: EncodingDetails for eager decoding
        - stderr_lines: result of calling splitlines() on the stderr output

        The child is run in isolated mode if the current interpreter supports
        that.
        """
        result, py_cmd = run_python_until_end('-X', 'utf8=0', '-c', cls.CHILD_PROCESS_SCRIPT, **env_vars)
        if not result.rc == 0:
            result.fail(py_cmd)
        stdout_lines = result.out.decode('ascii').splitlines()
        child_encoding_details = dict(cls(*stdout_lines)._asdict())
        stderr_lines = result.err.decode('ascii').rstrip().splitlines()
        return (child_encoding_details, stderr_lines)

LEGACY_LOCALE_WARNING = 'Python runtime initialized with LC_CTYPE=C (a locale with default ASCII encoding), which may cause Unicode compatibility problems. Using C.UTF-8, C.utf8, or UTF-8 (if available) as alternative Unicode-compatible locales is recommended.'

CLI_COERCION_WARNING_FMT = 'Python detected LC_CTYPE=C: LC_CTYPE coerced to {} (set another locale or PYTHONCOERCECLOCALE=0 to disable this locale coercion behavior).'

AVAILABLE_TARGETS = None

CLI_COERCION_TARGET = None

CLI_COERCION_WARNING = None

def setUpModule():
    global AVAILABLE_TARGETS
    global CLI_COERCION_TARGET
    global CLI_COERCION_WARNING
    if AVAILABLE_TARGETS is not None:
        return
    AVAILABLE_TARGETS = []
    for target_locale in _C_UTF8_LOCALES:
        if _set_locale_in_subprocess(target_locale):
            AVAILABLE_TARGETS.append(target_locale)
    if AVAILABLE_TARGETS:
        CLI_COERCION_TARGET = AVAILABLE_TARGETS[0]
        CLI_COERCION_WARNING = CLI_COERCION_WARNING_FMT.format(CLI_COERCION_TARGET)
    if support.verbose:
        print(f'AVAILABLE_TARGETS = {AVAILABLE_TARGETS!r}')
        print(f'EXPECTED_C_LOCALE_EQUIVALENTS = {EXPECTED_C_LOCALE_EQUIVALENTS!r}')
        print(f'EXPECTED_C_LOCALE_STREAM_ENCODING = {EXPECTED_C_LOCALE_STREAM_ENCODING!r}')
        print(f'EXPECTED_C_LOCALE_FS_ENCODING = {EXPECTED_C_LOCALE_FS_ENCODING!r}')
        print(f'EXPECT_COERCION_IN_DEFAULT_LOCALE = {EXPECT_COERCION_IN_DEFAULT_LOCALE!r}')
        print(f'_C_UTF8_LOCALES = {_C_UTF8_LOCALES!r}')
        print(f'_check_nl_langinfo_CODESET = {_check_nl_langinfo_CODESET!r}')

def tearDownModule():
    support.reap_children()


# --- test body ---
def _check_c_locale_coercion(fs_encoding, stream_encoding, coerce_c_locale, expected_warnings=None, coercion_expected=True, **extra_vars):
    """Check the C locale handling for various configurations

        Parameters:
            fs_encoding: expected sys.getfilesystemencoding() result
            stream_encoding: expected encoding for standard streams
            coerce_c_locale: setting to use for PYTHONCOERCECLOCALE
              None: don't set the variable at all
              str: the value set in the child's environment
            expected_warnings: expected warning lines on stderr
            extra_vars: additional environment variables to set in subprocess
        """
    self_maxDiff = None
    if not AVAILABLE_TARGETS:
        fs_encoding = EXPECTED_C_LOCALE_FS_ENCODING
        stream_encoding = EXPECTED_C_LOCALE_STREAM_ENCODING
        coercion_expected = False
        if expected_warnings:
            expected_warnings = [LEGACY_LOCALE_WARNING]
    base_var_dict = {'LANG': '', 'LC_CTYPE': '', 'LC_ALL': '', 'PYTHONCOERCECLOCALE': '', 'PYTHONIOENCODING': ''}
    base_var_dict.update(extra_vars)
    if coerce_c_locale is not None:
        base_var_dict['PYTHONCOERCECLOCALE'] = coerce_c_locale
    if EXPECT_COERCION_IN_DEFAULT_LOCALE:
        _expected_warnings = expected_warnings
        _coercion_expected = coercion_expected
    else:
        _expected_warnings = None
        _coercion_expected = False
    if support.is_android and _expected_warnings == [CLI_COERCION_WARNING]:
        _expected_warnings = None
    _check_child_encoding_details(base_var_dict, fs_encoding, stream_encoding, None, _expected_warnings, _coercion_expected)
    for locale_to_set in EXPECTED_C_LOCALE_EQUIVALENTS:
        for env_var in ('LANG', 'LC_CTYPE'):
            var_dict = base_var_dict.copy()
            var_dict[env_var] = locale_to_set
            _check_child_encoding_details(var_dict, fs_encoding, stream_encoding, None, expected_warnings, coercion_expected)

def _check_child_encoding_details(env_vars, expected_fs_encoding, expected_stream_encoding, expected_stream_errors, expected_warnings, coercion_expected):
    """Check the C locale handling for the given process environment

        Parameters:
            expected_fs_encoding: expected sys.getfilesystemencoding() result
            expected_stream_encoding: expected encoding for standard streams
            expected_warning: stderr output to expect (if any)
        """
    result = EncodingDetails.get_child_details(env_vars)
    encoding_details, stderr_lines = result
    expected_details = EncodingDetails.get_expected_details(coercion_expected, expected_fs_encoding, expected_stream_encoding, expected_stream_errors, env_vars)

    assert encoding_details == expected_details
    if expected_warnings is None:
        expected_warnings = []

    assert stderr_lines == expected_warnings
_check_c_locale_coercion(EXPECTED_C_LOCALE_FS_ENCODING, EXPECTED_C_LOCALE_STREAM_ENCODING, coerce_c_locale=None, LC_ALL='C', coercion_expected=False)
_check_c_locale_coercion(EXPECTED_C_LOCALE_FS_ENCODING, EXPECTED_C_LOCALE_STREAM_ENCODING, coerce_c_locale='warn', LC_ALL='C', expected_warnings=[LEGACY_LOCALE_WARNING], coercion_expected=False)
print("LocaleCoercionTests::test_LC_ALL_set_to_C: ok")
