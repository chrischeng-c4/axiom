# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regrtest"
# dimension = "behavior"
# case = "test_utils__test_sanitize_xml"
# subject = "cpython.test_regrtest.TestUtils.test_sanitize_xml"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_regrtest.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_regrtest.py::TestUtils::test_sanitize_xml
"""Auto-ported test: TestUtils::test_sanitize_xml (CPython 3.12 oracle)."""


import contextlib
import dataclasses
import glob
import io
import locale
import os.path
import platform
import random
import re
import shlex
import signal
import subprocess
import sys
import sysconfig
import tempfile
import textwrap
import unittest
from xml.etree import ElementTree
from test import support
from test.support import os_helper
from test.libregrtest import cmdline
from test.libregrtest import main
from test.libregrtest import setup
from test.libregrtest import utils
from test.libregrtest.filter import get_match_tests, set_match_tests, match_test
from test.libregrtest.result import TestStats
from test.libregrtest.utils import normalize_test_name


'\nTests of regrtest.py.\n\nNote: test_regrtest cannot be run twice in parallel.\n'

if not support.has_subprocess_support:
    raise unittest.SkipTest('test module requires subprocess')

ROOT_DIR = os.path.join(os.path.dirname(__file__), '..', '..')

ROOT_DIR = os.path.abspath(os.path.normpath(ROOT_DIR))

LOG_PREFIX = '[0-9]+:[0-9]+:[0-9]+ (?:load avg: [0-9]+\\.[0-9]{2} )?'

RESULT_REGEX = ('passed', 'failed', 'skipped', 'interrupted', 'env changed', 'timed out', 'ran no tests', 'worker non-zero exit code')

RESULT_REGEX = f"(?:{'|'.join(RESULT_REGEX)})"

EXITCODE_BAD_TEST = 2

EXITCODE_ENV_CHANGED = 3

EXITCODE_NO_TESTS_RAN = 4

EXITCODE_RERUN_FAIL = 5

EXITCODE_INTERRUPTED = 130

TEST_INTERRUPTED = textwrap.dedent('\n    from signal import SIGINT, raise_signal\n    try:\n        raise_signal(SIGINT)\n    except ImportError:\n        import os\n        os.kill(os.getpid(), SIGINT)\n    ')

@dataclasses.dataclass(slots=True)
class Rerun:
    name: str
    match: str | None
    success: bool

class BaseTestCase(unittest.TestCase):
    TEST_UNIQUE_ID = 1
    TESTNAME_PREFIX = 'test_regrtest_'
    TESTNAME_REGEX = 'test_[a-zA-Z0-9_]+'

    def setUp(self):
        self.testdir = os.path.realpath(os.path.dirname(__file__))
        self.tmptestdir = tempfile.mkdtemp()
        self.addCleanup(os_helper.rmtree, self.tmptestdir)

    def create_test(self, name=None, code=None):
        if not name:
            name = 'noop%s' % BaseTestCase.TEST_UNIQUE_ID
            BaseTestCase.TEST_UNIQUE_ID += 1
        if code is None:
            code = textwrap.dedent('\n                    import unittest\n\n                    class Tests(unittest.TestCase):\n                        def test_empty_test(self):\n                            pass\n                ')
        name = self.TESTNAME_PREFIX + name
        path = os.path.join(self.tmptestdir, name + '.py')
        self.addCleanup(os_helper.unlink, path)
        try:
            with open(path, 'x', encoding='utf-8') as fp:
                fp.write(code)
        except PermissionError as exc:
            if not sysconfig.is_python_build():
                self.skipTest('cannot write %s: %s' % (path, exc))
            raise
        return name

    def regex_search(self, regex, output):
        match = re.search(regex, output, re.MULTILINE)
        if not match:
            self.fail('%r not found in %r' % (regex, output))
        return match

    def check_line(self, output, pattern, full=False, regex=True):
        if not regex:
            pattern = re.escape(pattern)
        if full:
            pattern += '\n'
        regex = re.compile('^' + pattern, re.MULTILINE)
        self.assertRegex(output, regex)

    def parse_executed_tests(self, output):
        regex = f'^{LOG_PREFIX}\\[ *[0-9]+(?:/ *[0-9]+)*\\] ({self.TESTNAME_REGEX}) {RESULT_REGEX}'
        parser = re.finditer(regex, output, re.MULTILINE)
        return list((match.group(1) for match in parser))

    def check_executed_tests(self, output, tests, *, stats, skipped=(), failed=(), env_changed=(), omitted=(), rerun=None, run_no_tests=(), resource_denied=(), randomize=False, parallel=False, interrupted=False, fail_env_changed=False, forever=False, filtered=False):
        if isinstance(tests, str):
            tests = [tests]
        if isinstance(skipped, str):
            skipped = [skipped]
        if isinstance(resource_denied, str):
            resource_denied = [resource_denied]
        if isinstance(failed, str):
            failed = [failed]
        if isinstance(env_changed, str):
            env_changed = [env_changed]
        if isinstance(omitted, str):
            omitted = [omitted]
        if isinstance(run_no_tests, str):
            run_no_tests = [run_no_tests]
        if isinstance(stats, int):
            stats = TestStats(stats)
        if parallel:
            randomize = True
        rerun_failed = []
        if rerun is not None and (not env_changed):
            failed = [rerun.name]
            if not rerun.success:
                rerun_failed.append(rerun.name)
        executed = self.parse_executed_tests(output)
        total_tests = list(tests)
        if rerun is not None:
            total_tests.append(rerun.name)
        if randomize:
            self.assertEqual(set(executed), set(total_tests), output)
        else:
            self.assertEqual(executed, total_tests, output)

        def plural(count):
            return 's' if count != 1 else ''

        def list_regex(line_format, tests):
            count = len(tests)
            names = ' '.join(sorted(tests))
            regex = line_format % (count, plural(count))
            regex = '%s:\\n    %s$' % (regex, names)
            return regex
        if skipped:
            regex = list_regex('%s test%s skipped', skipped)
            self.check_line(output, regex)
        if resource_denied:
            regex = list_regex('%s test%s skipped \\(resource denied\\)', resource_denied)
            self.check_line(output, regex)
        if failed:
            regex = list_regex('%s test%s failed', failed)
            self.check_line(output, regex)
        if env_changed:
            regex = list_regex('%s test%s altered the execution environment \\(env changed\\)', env_changed)
            self.check_line(output, regex)
        if omitted:
            regex = list_regex('%s test%s omitted', omitted)
            self.check_line(output, regex)
        if rerun is not None:
            regex = list_regex('%s re-run test%s', [rerun.name])
            self.check_line(output, regex)
            regex = LOG_PREFIX + 'Re-running 1 failed tests in verbose mode'
            self.check_line(output, regex)
            regex = f'Re-running {rerun.name} in verbose mode'
            if rerun.match:
                regex = f'{regex} \\(matching: {rerun.match}\\)'
            self.check_line(output, regex)
        if run_no_tests:
            regex = list_regex('%s test%s run no tests', run_no_tests)
            self.check_line(output, regex)
        good = len(tests) - len(skipped) - len(resource_denied) - len(failed) - len(omitted) - len(env_changed) - len(run_no_tests)
        if good:
            regex = '%s test%s OK\\.' % (good, plural(good))
            if not skipped and (not failed) and (rerun is None or rerun.success) and (good > 1):
                regex = 'All %s' % regex
            self.check_line(output, regex, full=True)
        if interrupted:
            self.check_line(output, 'Test suite interrupted by signal SIGINT.')
        text = f'run={stats.tests_run:,}'
        if filtered:
            text = f'{text} \\(filtered\\)'
        parts = [text]
        if stats.failures:
            parts.append(f'failures={stats.failures:,}')
        if stats.skipped:
            parts.append(f'skipped={stats.skipped:,}')
        line = f"Total tests: {' '.join(parts)}"
        self.check_line(output, line, full=True)
        run = len(total_tests) - len(resource_denied)
        if rerun is not None:
            total_failed = len(rerun_failed)
            total_rerun = 1
        else:
            total_failed = len(failed)
            total_rerun = 0
        if interrupted:
            run = 0
        text = f'run={run}'
        if not forever:
            text = f'{text}/{len(tests)}'
        if filtered:
            text = f'{text} \\(filtered\\)'
        report = [text]
        for name, ntest in (('failed', total_failed), ('env_changed', len(env_changed)), ('skipped', len(skipped)), ('resource_denied', len(resource_denied)), ('rerun', total_rerun), ('run_no_tests', len(run_no_tests))):
            if ntest:
                report.append(f'{name}={ntest}')
        line = f"Total test files: {' '.join(report)}"
        self.check_line(output, line, full=True)
        state = []
        if failed:
            state.append('FAILURE')
        elif fail_env_changed and env_changed:
            state.append('ENV CHANGED')
        if interrupted:
            state.append('INTERRUPTED')
        if not any((good, failed, interrupted, skipped, env_changed, fail_env_changed)):
            state.append('NO TESTS RAN')
        elif not state:
            state.append('SUCCESS')
        state = ', '.join(state)
        if rerun is not None:
            new_state = 'SUCCESS' if rerun.success else 'FAILURE'
            state = f'{state} then {new_state}'
        self.check_line(output, f'Result: {state}', full=True)

    def parse_random_seed(self, output: str) -> str:
        match = self.regex_search('Using random seed: (.*)', output)
        return match.group(1)

    def run_command(self, args, input=None, exitcode=0, **kw):
        if not input:
            input = ''
        if 'stderr' not in kw:
            kw['stderr'] = subprocess.STDOUT
        env = kw.pop('env', None)
        if env is None:
            env = dict(os.environ)
            env.pop('SOURCE_DATE_EPOCH', None)
        proc = subprocess.run(args, text=True, input=input, stdout=subprocess.PIPE, env=env, **kw)
        if proc.returncode != exitcode:
            msg = 'Command %s failed with exit code %s, but exit code %s expected!\n\nstdout:\n---\n%s\n---\n' % (str(args), proc.returncode, exitcode, proc.stdout)
            if proc.stderr:
                msg += '\nstderr:\n---\n%s---\n' % proc.stderr
            self.fail(msg)
        return proc

    def run_python(self, args, **kw):
        extraargs = []
        if 'uops' in sys._xoptions:
            extraargs.extend(['-X', 'uops'])
        args = [sys.executable, *extraargs, '-X', 'faulthandler', '-I', *args]
        proc = self.run_command(args, **kw)
        return proc.stdout


# --- test body ---
sanitize_xml = utils.sanitize_xml

assert sanitize_xml('abc \x1b\x1f def') == 'abc \\x1b\\x1f def'

assert sanitize_xml('nul:\x00, bell:\x07') == 'nul:\\x00, bell:\\x07'

assert sanitize_xml('surrogate:\udc80') == 'surrogate:\\udc80'

assert sanitize_xml('illegal \ufffe and \uffff') == 'illegal \\ufffe and \\uffff'

assert sanitize_xml('a\n\tb') == 'a\n\tb'

assert sanitize_xml('valid téxt €') == 'valid téxt €'
print("TestUtils::test_sanitize_xml: ok")
