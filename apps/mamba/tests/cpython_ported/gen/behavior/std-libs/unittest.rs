use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/unittest/bare_testcase_runs_asserts.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_bare_testcase_runs_asserts() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "bare_testcase_runs_asserts"
# subject = "unittest.TestCase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestCase: a bare TestCase() can run individual asserts directly: assertEqual(3, 3) passes and a mismatch raises failureException inside assertRaises"""
import unittest

bare = unittest.TestCase()
bare.assertEqual(3, 3)
with bare.assertRaises(bare.failureException):
    bare.assertEqual(3, 2)
print("bare_testcase_runs_asserts OK")
"###);
    assert_output(&out, r###"bare_testcase_runs_asserts OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest/counttestcases_single_instance.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_counttestcases_single_instance() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "counttestcases_single_instance"
# subject = "unittest.TestCase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestCase: a single TestCase instance reports countTestCases() == 1"""
import unittest


class Sample(unittest.TestCase):
    def test_a(self):
        pass

    def runTest(self):
        pass


assert Sample("test_a").countTestCases() == 1
print("counttestcases_single_instance OK")
"###);
    assert_output(&out, r###"counttestcases_single_instance OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest/debug_runs_subtest_inline.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_debug_runs_subtest_inline() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "debug_runs_subtest_inline"
# subject = "unittest.TestCase.debug"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestCase.debug: debug() runs the test in-line and a passing subTest does not interrupt control flow"""
import unittest

events = []


class Debuggable(unittest.TestCase):
    def test_a(self):
        events.append("test case")
        with self.subTest():
            events.append("subtest 1")


Debuggable("test_a").debug()
assert events == ["test case", "subtest 1"]
print("debug_runs_subtest_inline OK")
"###);
    assert_output(&out, r###"debug_runs_subtest_inline OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest/failfast_stops_on_failing_subtest.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_failfast_stops_on_failing_subtest() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "failfast_stops_on_failing_subtest"
# subject = "unittest.TestResult"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestResult: under failfast, a failing subTest halts the suite immediately: test_a's subtests run, test_b fails on its second subTest, test_c never runs, and exactly one failure is recorded"""
import unittest

events = []


class FailFast(unittest.TestCase):
    def test_a(self):
        with self.subTest():
            events.append("a1")
        events.append("a2")

    def test_b(self):
        with self.subTest():
            events.append("b1")
        with self.subTest():
            self.fail("failure")
        events.append("b2")

    def test_c(self):
        events.append("c")


result = unittest.TestResult()
result.failfast = True
suite = unittest.TestLoader().loadTestsFromTestCase(FailFast)
suite.run(result)

assert events == ["a1", "a2", "b1"]
assert result.failfast is True
assert len(result.failures) == 1
print("failfast_stops_on_failing_subtest OK")
"###);
    assert_output(&out, r###"failfast_stops_on_failing_subtest OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest/failing_test_reports_addfailure.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_failing_test_reports_addfailure() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "failing_test_reports_addfailure"
# subject = "unittest.TestResult"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestResult: a failing test reports addFailure (not addError) between startTest/stopTest, and a custom failureException is honored for self.fail()"""
import unittest

events = []


class RecordingResult(unittest.TestResult):
    def startTest(self, test):
        events.append("startTest")
        super().startTest(test)

    def addFailure(self, test, err):
        events.append("addFailure")
        super().addFailure(test, err)

    def stopTest(self, test):
        events.append("stopTest")
        super().stopTest(test)


class Failing(unittest.TestCase):
    failureException = RuntimeError

    def test(self):
        self.fail("boom")


assert Failing("test").failureException is RuntimeError
Failing("test").run(RecordingResult())
assert events == ["startTest", "addFailure", "stopTest"]
print("failing_test_reports_addfailure OK")
"###);
    assert_output(&out, r###"failing_test_reports_addfailure OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest/loadtestsfromname_missing_name_yields_failing_test.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_loadtestsfromname_missing_name_yields_failing_test() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "loadtestsfromname_missing_name_yields_failing_test"
# subject = "unittest.TestLoader.loadTestsFromName"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestLoader.loadTestsFromName: loadTestsFromName for a non-existent top-level name does not raise; it returns a one-test suite that records an error when run"""
import unittest

loader = unittest.TestLoader()
# CPython 3.12 defers the failure: building the suite does not raise.
suite = loader.loadTestsFromName("no_such_module_xyz")
assert suite.countTestCases() == 1
result = unittest.TestResult()
suite.run(result)
# The deferred failure surfaces as an error when the suite runs.
assert len(result.errors) == 1
assert result.failures == []
print("loadtestsfromname_missing_name_yields_failing_test OK")
"###);
    assert_output(&out, r###"loadtestsfromname_missing_name_yields_failing_test OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest/passing_run_records_no_failures.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_passing_run_records_no_failures() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "passing_run_records_no_failures"
# subject = "unittest.TestResult"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestResult: after a passing run, the result has testsRun == 1, empty failures/errors, and wasSuccessful() is True"""
import unittest


class Passing(unittest.TestCase):
    def test(self):
        pass


given = unittest.TestResult()
Passing("test").run(given)
assert given.testsRun == 1
assert given.failures == []
assert given.errors == []
assert given.wasSuccessful()
print("passing_run_records_no_failures OK")
"###);
    assert_output(&out, r###"passing_run_records_no_failures OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest/passing_test_event_ordering.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_passing_test_event_ordering() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "passing_test_event_ordering"
# subject = "unittest.TestResult"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestResult: a passing test run via defaultTestResult() brackets the run with startTestRun/stopTestRun and reports addSuccess: startTestRun, startTest, body, addSuccess, stopTest, stopTestRun"""
import unittest

events = []


class RecordingResult(unittest.TestResult):
    def startTestRun(self):
        events.append("startTestRun")
        super().startTestRun()

    def startTest(self, test):
        events.append("startTest")
        super().startTest(test)

    def addSuccess(self, test):
        events.append("addSuccess")
        super().addSuccess(test)

    def stopTest(self, test):
        events.append("stopTest")
        super().stopTest(test)

    def stopTestRun(self):
        events.append("stopTestRun")
        super().stopTestRun()


default_result = RecordingResult()


class Passing(unittest.TestCase):
    def test(self):
        events.append("body")

    def defaultTestResult(self):
        return default_result


assert Passing("test").run() is default_result
assert events == [
    "startTestRun",
    "startTest",
    "body",
    "addSuccess",
    "stopTest",
    "stopTestRun",
]
print("passing_test_event_ordering OK")
"###);
    assert_output(&out, r###"passing_test_event_ordering OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest/run_returns_given_result.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_run_returns_given_result() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "run_returns_given_result"
# subject = "unittest.TestCase.run"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestCase.run: TestCase.run(result) returns the same TestResult object it was handed"""
import unittest


class Passing(unittest.TestCase):
    def test(self):
        pass


given = unittest.TestResult()
returned = Passing("test").run(given)
assert returned is given
print("run_returns_given_result OK")
"###);
    assert_output(&out, r###"run_returns_given_result OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest/run_uses_default_result.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_run_uses_default_result() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "run_uses_default_result"
# subject = "unittest.TestCase.run"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestCase.run: TestCase.run() with no argument falls back to the object's defaultTestResult()"""
import unittest

default_result = unittest.TestResult()


class WithDefault(unittest.TestCase):
    def test(self):
        pass

    def defaultTestResult(self):
        return default_result


used = WithDefault("test").run()
assert used is default_result
print("run_uses_default_result OK")
"###);
    assert_output(&out, r###"run_uses_default_result OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest/setup_teardown_default_noops.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_setup_teardown_default_noops() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "setup_teardown_default_noops"
# subject = "unittest.TestCase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestCase: the default setUp() and tearDown() are safe no-ops callable directly without error"""
import unittest


class Sample(unittest.TestCase):
    def test_a(self):
        pass

    def runTest(self):
        pass


assert Sample().setUp() is None
assert Sample().tearDown() is None
print("setup_teardown_default_noops OK")
"###);
    assert_output(&out, r###"setup_teardown_default_noops OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest/testresult_initial_counters_empty.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_testresult_initial_counters_empty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "testresult_initial_counters_empty"
# subject = "unittest.TestResult"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestResult: a fresh TestResult starts with empty failures and errors lists"""
import unittest

result = unittest.TestResult()
assert result.failures == []
assert result.errors == []
print("testresult_initial_counters_empty OK")
"###);
    assert_output(&out, r###"testresult_initial_counters_empty OK
"###);
}
