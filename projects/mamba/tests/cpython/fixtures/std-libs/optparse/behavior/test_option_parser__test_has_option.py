# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "behavior"
# case = "test_option_parser__test_has_option"
# subject = "cpython.test_optparse.TestOptionParser.test_has_option"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_optparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_optparse.py::TestOptionParser::test_has_option
"""Auto-ported test: TestOptionParser::test_has_option (CPython 3.12 oracle)."""


import sys
import os
import re
import copy
import unittest
from io import StringIO
from test import support
from test.support import os_helper
from test.support.i18n_helper import TestTranslationsBase, update_translation_snapshots
import optparse
from optparse import make_option, Option, TitledHelpFormatter, OptionParser, OptionGroup, SUPPRESS_USAGE, OptionError, OptionConflictError, BadOptionError, OptionValueError, Values
from optparse import _match_abbrev
from optparse import _parse_num


class InterceptedError(Exception):

    def __init__(self, error_message=None, exit_status=None, exit_message=None):
        self.error_message = error_message
        self.exit_status = exit_status
        self.exit_message = exit_message

    def __str__(self):
        return self.error_message or self.exit_message or 'intercepted error'

class InterceptingOptionParser(OptionParser):

    def exit(self, status=0, msg=None):
        raise InterceptedError(exit_status=status, exit_message=msg)

    def error(self, msg):
        raise InterceptedError(error_message=msg)

class BaseTest(unittest.TestCase):

    def assertParseOK(self, args, expected_opts, expected_positional_args):
        """Assert the options are what we expected when parsing arguments.

        Otherwise, fail with a nicely formatted message.

        Keyword arguments:
        args -- A list of arguments to parse with OptionParser.
        expected_opts -- The options expected.
        expected_positional_args -- The positional arguments expected.

        Returns the options and positional args for further testing.
        """
        options, positional_args = self.parser.parse_args(args)
        optdict = vars(options)
        self.assertEqual(optdict, expected_opts, '\nOptions are %(optdict)s.\nShould be %(expected_opts)s.\nArgs were %(args)s.' % locals())
        self.assertEqual(positional_args, expected_positional_args, '\nPositional arguments are %(positional_args)s.\nShould be %(expected_positional_args)s.\nArgs were %(args)s.' % locals())
        return (options, positional_args)

    def assertRaises(self, func, args, kwargs, expected_exception, expected_message):
        """
        Assert that the expected exception is raised when calling a
        function, and that the right error message is included with
        that exception.

        Arguments:
          func -- the function to call
          args -- positional arguments to `func`
          kwargs -- keyword arguments to `func`
          expected_exception -- exception that should be raised
          expected_message -- expected exception message (or pattern
            if a compiled regex object)

        Returns the exception raised for further testing.
        """
        if args is None:
            args = ()
        if kwargs is None:
            kwargs = {}
        try:
            func(*args, **kwargs)
        except expected_exception as err:
            actual_message = str(err)
            if isinstance(expected_message, re.Pattern):
                self.assertTrue(expected_message.search(actual_message), "expected exception message pattern:\n/%s/\nactual exception message:\n'''%s'''\n" % (expected_message.pattern, actual_message))
            else:
                self.assertEqual(actual_message, expected_message, "expected exception message:\n'''%s'''\nactual exception message:\n'''%s'''\n" % (expected_message, actual_message))
            return err
        else:
            self.fail('expected exception %(expected_exception)s not raised\ncalled %(func)r\nwith args %(args)r\nand kwargs %(kwargs)r\n' % locals())

    def assertParseFail(self, cmdline_args, expected_output):
        """
        Assert the parser fails with the expected message.  Caller
        must ensure that self.parser is an InterceptingOptionParser.
        """
        try:
            self.parser.parse_args(cmdline_args)
        except InterceptedError as err:
            self.assertEqual(err.error_message, expected_output)
        else:
            self.assertFalse('expected parse failure')

    def assertOutput(self, cmdline_args, expected_output, expected_status=0, expected_error=None):
        """Assert the parser prints the expected output on stdout."""
        save_stdout = sys.stdout
        try:
            try:
                sys.stdout = StringIO()
                self.parser.parse_args(cmdline_args)
            finally:
                output = sys.stdout.getvalue()
                sys.stdout = save_stdout
        except InterceptedError as err:
            self.assertTrue(isinstance(output, str), 'expected output to be an ordinary string, not %r' % type(output))
            if output != expected_output:
                self.fail("expected: \n'''\n" + expected_output + "'''\nbut got \n'''\n" + output + "'''")
            self.assertEqual(err.exit_status, expected_status)
            self.assertEqual(err.exit_message, expected_error)
        else:
            self.assertFalse('expected parser.exit()')

    def assertTypeError(self, func, expected_message, *args):
        """Assert that TypeError is raised when executing func."""
        self.assertRaises(func, args, None, TypeError, expected_message)

    def assertHelp(self, parser, expected_help):
        actual_help = parser.format_help()
        if actual_help != expected_help:
            raise self.failureException('help text failure; expected:\n"' + expected_help + '"; got:\n"' + actual_help + '"\n')

_time_units = {'s': 1, 'm': 60, 'h': 60 * 60, 'd': 60 * 60 * 24}

def _check_duration(option, opt, value):
    try:
        if value[-1].isdigit():
            return int(value)
        else:
            return int(value[:-1]) * _time_units[value[-1]]
    except (ValueError, IndexError):
        raise OptionValueError('option %s: invalid duration: %r' % (opt, value))

class DurationOption(Option):
    TYPES = Option.TYPES + ('duration',)
    TYPE_CHECKER = copy.copy(Option.TYPE_CHECKER)
    TYPE_CHECKER['duration'] = _check_duration

class ConflictBase(BaseTest):

    def setUp(self):
        options = [make_option('-v', '--verbose', action='count', dest='verbose', help='increment verbosity')]
        self.parser = InterceptingOptionParser(usage=SUPPRESS_USAGE, option_list=options)

    def show_version(self, option, opt, value, parser):
        parser.values.show_version = 1

_expected_help_basic = 'Usage: bar.py [options]\n\nOptions:\n  -a APPLE           throw APPLEs at basket\n  -b NUM, --boo=NUM  shout "boo!" NUM times (in order to frighten away all the\n                     evil spirits that cause trouble and mayhem)\n  --foo=FOO          store FOO in the foo list for later fooing\n  -h, --help         show this help message and exit\n'

_expected_help_long_opts_first = 'Usage: bar.py [options]\n\nOptions:\n  -a APPLE           throw APPLEs at basket\n  --boo=NUM, -b NUM  shout "boo!" NUM times (in order to frighten away all the\n                     evil spirits that cause trouble and mayhem)\n  --foo=FOO          store FOO in the foo list for later fooing\n  --help, -h         show this help message and exit\n'

_expected_help_title_formatter = 'Usage\n=====\n  bar.py [options]\n\nOptions\n=======\n-a APPLE           throw APPLEs at basket\n--boo=NUM, -b NUM  shout "boo!" NUM times (in order to frighten away all the\n                   evil spirits that cause trouble and mayhem)\n--foo=FOO          store FOO in the foo list for later fooing\n--help, -h         show this help message and exit\n'

_expected_help_short_lines = 'Usage: bar.py [options]\n\nOptions:\n  -a APPLE           throw APPLEs at basket\n  -b NUM, --boo=NUM  shout "boo!" NUM times (in order to\n                     frighten away all the evil spirits\n                     that cause trouble and mayhem)\n  --foo=FOO          store FOO in the foo list for later\n                     fooing\n  -h, --help         show this help message and exit\n'

_expected_very_help_short_lines = 'Usage: bar.py [options]\n\nOptions:\n  -a APPLE\n    throw\n    APPLEs at\n    basket\n  -b NUM, --boo=NUM\n    shout\n    "boo!" NUM\n    times (in\n    order to\n    frighten\n    away all\n    the evil\n    spirits\n    that cause\n    trouble and\n    mayhem)\n  --foo=FOO\n    store FOO\n    in the foo\n    list for\n    later\n    fooing\n  -h, --help\n    show this\n    help\n    message and\n    exit\n'


# --- test body ---
self_parser = OptionParser()
self_parser.add_option('-v', '--verbose', '-n', '--noisy', action='store_true', dest='verbose')
self_parser.add_option('-q', '--quiet', '--silent', action='store_false', dest='verbose')

assert self_parser.has_option('-v')

assert self_parser.has_option('--verbose')
print("TestOptionParser::test_has_option: ok")
