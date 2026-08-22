use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/core/exception_group/exception_group_subgroup_tests__test_basics_subgroup_by_predicate_no_match.py`.
#[test]
fn test_gen_behavior_core_exception_group_exception_group_subgroup_tests__test_basics_subgroup_by_predicate_no_match() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exception_group"
# dimension = "behavior"
# case = "exception_group_subgroup_tests__test_basics_subgroup_by_predicate_no_match"
# subject = "cpython.test_exception_group.ExceptionGroupSubgroupTests.test_basics_subgroup_by_predicate__no_match"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_exception_group.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_group.py::ExceptionGroupSubgroupTests::test_basics_subgroup_by_predicate__no_match
"""Auto-ported test: ExceptionGroupSubgroupTests::test_basics_subgroup_by_predicate__no_match (CPython 3.12 oracle)."""


import collections.abc
import types
import unittest
from test.support import C_RECURSION_LIMIT


def create_simple_eg():
    excs = []
    try:
        try:
            raise MemoryError('context and cause for ValueError(1)')
        except MemoryError as e:
            raise ValueError(1) from e
    except ValueError as e:
        excs.append(e)
    try:
        try:
            raise OSError('context for TypeError')
        except OSError as e:
            raise TypeError(int)
    except TypeError as e:
        excs.append(e)
    try:
        try:
            raise ImportError('context for ValueError(2)')
        except ImportError as e:
            raise ValueError(2)
    except ValueError as e:
        excs.append(e)
    try:
        raise ExceptionGroup('simple eg', excs)
    except ExceptionGroup as e:
        return e

class ExceptionGroupTestBase(unittest.TestCase):

    def assertMatchesTemplate(self, exc, exc_type, template):
        """ Assert that the exception matches the template

            A template describes the shape of exc. If exc is a
            leaf exception (i.e., not an exception group) then
            template is an exception instance that has the
            expected type and args value of exc. If exc is an
            exception group, then template is a list of the
            templates of its nested exceptions.
        """
        if exc_type is not None:
            self.assertIs(type(exc), exc_type)
        if isinstance(exc, BaseExceptionGroup):
            self.assertIsInstance(template, collections.abc.Sequence)
            self.assertEqual(len(exc.exceptions), len(template))
            for e, t in zip(exc.exceptions, template):
                self.assertMatchesTemplate(e, None, t)
        else:
            self.assertIsInstance(template, BaseException)
            self.assertEqual(type(exc), type(template))
            self.assertEqual(exc.args, template.args)

def leaf_generator(exc, tbs=None):
    if tbs is None:
        tbs = []
    tbs.append(exc.__traceback__)
    if isinstance(exc, BaseExceptionGroup):
        for e in exc.exceptions:
            yield from leaf_generator(e, tbs)
    else:
        yield (exc, tbs)
    tbs.pop()

def create_nested_eg():
    excs = []
    try:
        try:
            raise TypeError(bytes)
        except TypeError as e:
            raise ExceptionGroup('nested', [e])
    except ExceptionGroup as e:
        excs.append(e)
    try:
        try:
            raise MemoryError('out of memory')
        except MemoryError as e:
            raise ValueError(1) from e
    except ValueError as e:
        excs.append(e)
    try:
        raise ExceptionGroup('root', excs)
    except ExceptionGroup as eg:
        return eg

class ExceptionGroupSplitTestBase(ExceptionGroupTestBase):

    def split_exception_group(self, eg, types):
        """ Split an EG and do some sanity checks on the result """
        self.assertIsInstance(eg, BaseExceptionGroup)
        match, rest = eg.split(types)
        sg = eg.subgroup(types)
        if match is not None:
            self.assertIsInstance(match, BaseExceptionGroup)
            for e, _ in leaf_generator(match):
                self.assertIsInstance(e, types)
            self.assertIsNotNone(sg)
            self.assertIsInstance(sg, BaseExceptionGroup)
            for e, _ in leaf_generator(sg):
                self.assertIsInstance(e, types)
        if rest is not None:
            self.assertIsInstance(rest, BaseExceptionGroup)

        def leaves(exc):
            return [] if exc is None else [e for e, _ in leaf_generator(exc)]
        self.assertSequenceEqual(leaves(match), leaves(sg))
        match_leaves = leaves(match)
        rest_leaves = leaves(rest)
        self.assertEqual(len(leaves(eg)), len(leaves(match)) + len(leaves(rest)))
        for e in leaves(eg):
            self.assertNotEqual(match and e in match_leaves, rest and e in rest_leaves)
        for part in [match, rest, sg]:
            if part is not None:
                self.assertEqual(eg.message, part.message)
                self.assertIs(eg.__cause__, part.__cause__)
                self.assertIs(eg.__context__, part.__context__)
                self.assertIs(eg.__traceback__, part.__traceback__)
                self.assertEqual(getattr(eg, '__notes__', None), getattr(part, '__notes__', None))

        def tbs_for_leaf(leaf, eg):
            for e, tbs in leaf_generator(eg):
                if e is leaf:
                    return tbs

        def tb_linenos(tbs):
            return [tb.tb_lineno for tb in tbs if tb]
        for part in [match, rest, sg]:
            for e in leaves(part):
                self.assertSequenceEqual(tb_linenos(tbs_for_leaf(e, eg)), tb_linenos(tbs_for_leaf(e, part)))
        return (match, rest)


# --- test body ---
self_eg = create_simple_eg()
self_eg_template = [ValueError(1), TypeError(int), ValueError(2)]

assert self_eg.subgroup(lambda e: False) is None
print("ExceptionGroupSubgroupTests::test_basics_subgroup_by_predicate__no_match: ok")
"###);
    assert_output(&out, r###"ExceptionGroupSubgroupTests::test_basics_subgroup_by_predicate__no_match: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/exception_group/exception_group_subgroup_tests__test_basics_subgroup_by_predicate_passthrough.py`.
#[test]
fn test_gen_behavior_core_exception_group_exception_group_subgroup_tests__test_basics_subgroup_by_predicate_passthrough() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exception_group"
# dimension = "behavior"
# case = "exception_group_subgroup_tests__test_basics_subgroup_by_predicate_passthrough"
# subject = "cpython.test_exception_group.ExceptionGroupSubgroupTests.test_basics_subgroup_by_predicate__passthrough"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_exception_group.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_group.py::ExceptionGroupSubgroupTests::test_basics_subgroup_by_predicate__passthrough
"""Auto-ported test: ExceptionGroupSubgroupTests::test_basics_subgroup_by_predicate__passthrough (CPython 3.12 oracle)."""


import collections.abc
import types
import unittest
from test.support import C_RECURSION_LIMIT


def create_simple_eg():
    excs = []
    try:
        try:
            raise MemoryError('context and cause for ValueError(1)')
        except MemoryError as e:
            raise ValueError(1) from e
    except ValueError as e:
        excs.append(e)
    try:
        try:
            raise OSError('context for TypeError')
        except OSError as e:
            raise TypeError(int)
    except TypeError as e:
        excs.append(e)
    try:
        try:
            raise ImportError('context for ValueError(2)')
        except ImportError as e:
            raise ValueError(2)
    except ValueError as e:
        excs.append(e)
    try:
        raise ExceptionGroup('simple eg', excs)
    except ExceptionGroup as e:
        return e

class ExceptionGroupTestBase(unittest.TestCase):

    def assertMatchesTemplate(self, exc, exc_type, template):
        """ Assert that the exception matches the template

            A template describes the shape of exc. If exc is a
            leaf exception (i.e., not an exception group) then
            template is an exception instance that has the
            expected type and args value of exc. If exc is an
            exception group, then template is a list of the
            templates of its nested exceptions.
        """
        if exc_type is not None:
            self.assertIs(type(exc), exc_type)
        if isinstance(exc, BaseExceptionGroup):
            self.assertIsInstance(template, collections.abc.Sequence)
            self.assertEqual(len(exc.exceptions), len(template))
            for e, t in zip(exc.exceptions, template):
                self.assertMatchesTemplate(e, None, t)
        else:
            self.assertIsInstance(template, BaseException)
            self.assertEqual(type(exc), type(template))
            self.assertEqual(exc.args, template.args)

def leaf_generator(exc, tbs=None):
    if tbs is None:
        tbs = []
    tbs.append(exc.__traceback__)
    if isinstance(exc, BaseExceptionGroup):
        for e in exc.exceptions:
            yield from leaf_generator(e, tbs)
    else:
        yield (exc, tbs)
    tbs.pop()

def create_nested_eg():
    excs = []
    try:
        try:
            raise TypeError(bytes)
        except TypeError as e:
            raise ExceptionGroup('nested', [e])
    except ExceptionGroup as e:
        excs.append(e)
    try:
        try:
            raise MemoryError('out of memory')
        except MemoryError as e:
            raise ValueError(1) from e
    except ValueError as e:
        excs.append(e)
    try:
        raise ExceptionGroup('root', excs)
    except ExceptionGroup as eg:
        return eg

class ExceptionGroupSplitTestBase(ExceptionGroupTestBase):

    def split_exception_group(self, eg, types):
        """ Split an EG and do some sanity checks on the result """
        self.assertIsInstance(eg, BaseExceptionGroup)
        match, rest = eg.split(types)
        sg = eg.subgroup(types)
        if match is not None:
            self.assertIsInstance(match, BaseExceptionGroup)
            for e, _ in leaf_generator(match):
                self.assertIsInstance(e, types)
            self.assertIsNotNone(sg)
            self.assertIsInstance(sg, BaseExceptionGroup)
            for e, _ in leaf_generator(sg):
                self.assertIsInstance(e, types)
        if rest is not None:
            self.assertIsInstance(rest, BaseExceptionGroup)

        def leaves(exc):
            return [] if exc is None else [e for e, _ in leaf_generator(exc)]
        self.assertSequenceEqual(leaves(match), leaves(sg))
        match_leaves = leaves(match)
        rest_leaves = leaves(rest)
        self.assertEqual(len(leaves(eg)), len(leaves(match)) + len(leaves(rest)))
        for e in leaves(eg):
            self.assertNotEqual(match and e in match_leaves, rest and e in rest_leaves)
        for part in [match, rest, sg]:
            if part is not None:
                self.assertEqual(eg.message, part.message)
                self.assertIs(eg.__cause__, part.__cause__)
                self.assertIs(eg.__context__, part.__context__)
                self.assertIs(eg.__traceback__, part.__traceback__)
                self.assertEqual(getattr(eg, '__notes__', None), getattr(part, '__notes__', None))

        def tbs_for_leaf(leaf, eg):
            for e, tbs in leaf_generator(eg):
                if e is leaf:
                    return tbs

        def tb_linenos(tbs):
            return [tb.tb_lineno for tb in tbs if tb]
        for part in [match, rest, sg]:
            for e in leaves(part):
                self.assertSequenceEqual(tb_linenos(tbs_for_leaf(e, eg)), tb_linenos(tbs_for_leaf(e, part)))
        return (match, rest)


# --- test body ---
self_eg = create_simple_eg()
self_eg_template = [ValueError(1), TypeError(int), ValueError(2)]

assert self_eg is self_eg.subgroup(lambda e: True)
print("ExceptionGroupSubgroupTests::test_basics_subgroup_by_predicate__passthrough: ok")
"###);
    assert_output(&out, r###"ExceptionGroupSubgroupTests::test_basics_subgroup_by_predicate__passthrough: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/exception_group/exception_group_subgroup_tests__test_basics_subgroup_by_type_no_match.py`.
#[test]
fn test_gen_behavior_core_exception_group_exception_group_subgroup_tests__test_basics_subgroup_by_type_no_match() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exception_group"
# dimension = "behavior"
# case = "exception_group_subgroup_tests__test_basics_subgroup_by_type_no_match"
# subject = "cpython.test_exception_group.ExceptionGroupSubgroupTests.test_basics_subgroup_by_type__no_match"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_exception_group.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_group.py::ExceptionGroupSubgroupTests::test_basics_subgroup_by_type__no_match
"""Auto-ported test: ExceptionGroupSubgroupTests::test_basics_subgroup_by_type__no_match (CPython 3.12 oracle)."""


import collections.abc
import types
import unittest
from test.support import C_RECURSION_LIMIT


def create_simple_eg():
    excs = []
    try:
        try:
            raise MemoryError('context and cause for ValueError(1)')
        except MemoryError as e:
            raise ValueError(1) from e
    except ValueError as e:
        excs.append(e)
    try:
        try:
            raise OSError('context for TypeError')
        except OSError as e:
            raise TypeError(int)
    except TypeError as e:
        excs.append(e)
    try:
        try:
            raise ImportError('context for ValueError(2)')
        except ImportError as e:
            raise ValueError(2)
    except ValueError as e:
        excs.append(e)
    try:
        raise ExceptionGroup('simple eg', excs)
    except ExceptionGroup as e:
        return e

class ExceptionGroupTestBase(unittest.TestCase):

    def assertMatchesTemplate(self, exc, exc_type, template):
        """ Assert that the exception matches the template

            A template describes the shape of exc. If exc is a
            leaf exception (i.e., not an exception group) then
            template is an exception instance that has the
            expected type and args value of exc. If exc is an
            exception group, then template is a list of the
            templates of its nested exceptions.
        """
        if exc_type is not None:
            self.assertIs(type(exc), exc_type)
        if isinstance(exc, BaseExceptionGroup):
            self.assertIsInstance(template, collections.abc.Sequence)
            self.assertEqual(len(exc.exceptions), len(template))
            for e, t in zip(exc.exceptions, template):
                self.assertMatchesTemplate(e, None, t)
        else:
            self.assertIsInstance(template, BaseException)
            self.assertEqual(type(exc), type(template))
            self.assertEqual(exc.args, template.args)

def leaf_generator(exc, tbs=None):
    if tbs is None:
        tbs = []
    tbs.append(exc.__traceback__)
    if isinstance(exc, BaseExceptionGroup):
        for e in exc.exceptions:
            yield from leaf_generator(e, tbs)
    else:
        yield (exc, tbs)
    tbs.pop()

def create_nested_eg():
    excs = []
    try:
        try:
            raise TypeError(bytes)
        except TypeError as e:
            raise ExceptionGroup('nested', [e])
    except ExceptionGroup as e:
        excs.append(e)
    try:
        try:
            raise MemoryError('out of memory')
        except MemoryError as e:
            raise ValueError(1) from e
    except ValueError as e:
        excs.append(e)
    try:
        raise ExceptionGroup('root', excs)
    except ExceptionGroup as eg:
        return eg

class ExceptionGroupSplitTestBase(ExceptionGroupTestBase):

    def split_exception_group(self, eg, types):
        """ Split an EG and do some sanity checks on the result """
        self.assertIsInstance(eg, BaseExceptionGroup)
        match, rest = eg.split(types)
        sg = eg.subgroup(types)
        if match is not None:
            self.assertIsInstance(match, BaseExceptionGroup)
            for e, _ in leaf_generator(match):
                self.assertIsInstance(e, types)
            self.assertIsNotNone(sg)
            self.assertIsInstance(sg, BaseExceptionGroup)
            for e, _ in leaf_generator(sg):
                self.assertIsInstance(e, types)
        if rest is not None:
            self.assertIsInstance(rest, BaseExceptionGroup)

        def leaves(exc):
            return [] if exc is None else [e for e, _ in leaf_generator(exc)]
        self.assertSequenceEqual(leaves(match), leaves(sg))
        match_leaves = leaves(match)
        rest_leaves = leaves(rest)
        self.assertEqual(len(leaves(eg)), len(leaves(match)) + len(leaves(rest)))
        for e in leaves(eg):
            self.assertNotEqual(match and e in match_leaves, rest and e in rest_leaves)
        for part in [match, rest, sg]:
            if part is not None:
                self.assertEqual(eg.message, part.message)
                self.assertIs(eg.__cause__, part.__cause__)
                self.assertIs(eg.__context__, part.__context__)
                self.assertIs(eg.__traceback__, part.__traceback__)
                self.assertEqual(getattr(eg, '__notes__', None), getattr(part, '__notes__', None))

        def tbs_for_leaf(leaf, eg):
            for e, tbs in leaf_generator(eg):
                if e is leaf:
                    return tbs

        def tb_linenos(tbs):
            return [tb.tb_lineno for tb in tbs if tb]
        for part in [match, rest, sg]:
            for e in leaves(part):
                self.assertSequenceEqual(tb_linenos(tbs_for_leaf(e, eg)), tb_linenos(tbs_for_leaf(e, part)))
        return (match, rest)


# --- test body ---
self_eg = create_simple_eg()
self_eg_template = [ValueError(1), TypeError(int), ValueError(2)]

assert self_eg.subgroup(OSError) is None
print("ExceptionGroupSubgroupTests::test_basics_subgroup_by_type__no_match: ok")
"###);
    assert_output(&out, r###"ExceptionGroupSubgroupTests::test_basics_subgroup_by_type__no_match: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/exception_group/exception_group_subgroup_tests__test_basics_subgroup_by_type_passthrough.py`.
#[test]
fn test_gen_behavior_core_exception_group_exception_group_subgroup_tests__test_basics_subgroup_by_type_passthrough() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exception_group"
# dimension = "behavior"
# case = "exception_group_subgroup_tests__test_basics_subgroup_by_type_passthrough"
# subject = "cpython.test_exception_group.ExceptionGroupSubgroupTests.test_basics_subgroup_by_type__passthrough"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_exception_group.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_group.py::ExceptionGroupSubgroupTests::test_basics_subgroup_by_type__passthrough
"""Auto-ported test: ExceptionGroupSubgroupTests::test_basics_subgroup_by_type__passthrough (CPython 3.12 oracle)."""


import collections.abc
import types
import unittest
from test.support import C_RECURSION_LIMIT


def create_simple_eg():
    excs = []
    try:
        try:
            raise MemoryError('context and cause for ValueError(1)')
        except MemoryError as e:
            raise ValueError(1) from e
    except ValueError as e:
        excs.append(e)
    try:
        try:
            raise OSError('context for TypeError')
        except OSError as e:
            raise TypeError(int)
    except TypeError as e:
        excs.append(e)
    try:
        try:
            raise ImportError('context for ValueError(2)')
        except ImportError as e:
            raise ValueError(2)
    except ValueError as e:
        excs.append(e)
    try:
        raise ExceptionGroup('simple eg', excs)
    except ExceptionGroup as e:
        return e

class ExceptionGroupTestBase(unittest.TestCase):

    def assertMatchesTemplate(self, exc, exc_type, template):
        """ Assert that the exception matches the template

            A template describes the shape of exc. If exc is a
            leaf exception (i.e., not an exception group) then
            template is an exception instance that has the
            expected type and args value of exc. If exc is an
            exception group, then template is a list of the
            templates of its nested exceptions.
        """
        if exc_type is not None:
            self.assertIs(type(exc), exc_type)
        if isinstance(exc, BaseExceptionGroup):
            self.assertIsInstance(template, collections.abc.Sequence)
            self.assertEqual(len(exc.exceptions), len(template))
            for e, t in zip(exc.exceptions, template):
                self.assertMatchesTemplate(e, None, t)
        else:
            self.assertIsInstance(template, BaseException)
            self.assertEqual(type(exc), type(template))
            self.assertEqual(exc.args, template.args)

def leaf_generator(exc, tbs=None):
    if tbs is None:
        tbs = []
    tbs.append(exc.__traceback__)
    if isinstance(exc, BaseExceptionGroup):
        for e in exc.exceptions:
            yield from leaf_generator(e, tbs)
    else:
        yield (exc, tbs)
    tbs.pop()

def create_nested_eg():
    excs = []
    try:
        try:
            raise TypeError(bytes)
        except TypeError as e:
            raise ExceptionGroup('nested', [e])
    except ExceptionGroup as e:
        excs.append(e)
    try:
        try:
            raise MemoryError('out of memory')
        except MemoryError as e:
            raise ValueError(1) from e
    except ValueError as e:
        excs.append(e)
    try:
        raise ExceptionGroup('root', excs)
    except ExceptionGroup as eg:
        return eg

class ExceptionGroupSplitTestBase(ExceptionGroupTestBase):

    def split_exception_group(self, eg, types):
        """ Split an EG and do some sanity checks on the result """
        self.assertIsInstance(eg, BaseExceptionGroup)
        match, rest = eg.split(types)
        sg = eg.subgroup(types)
        if match is not None:
            self.assertIsInstance(match, BaseExceptionGroup)
            for e, _ in leaf_generator(match):
                self.assertIsInstance(e, types)
            self.assertIsNotNone(sg)
            self.assertIsInstance(sg, BaseExceptionGroup)
            for e, _ in leaf_generator(sg):
                self.assertIsInstance(e, types)
        if rest is not None:
            self.assertIsInstance(rest, BaseExceptionGroup)

        def leaves(exc):
            return [] if exc is None else [e for e, _ in leaf_generator(exc)]
        self.assertSequenceEqual(leaves(match), leaves(sg))
        match_leaves = leaves(match)
        rest_leaves = leaves(rest)
        self.assertEqual(len(leaves(eg)), len(leaves(match)) + len(leaves(rest)))
        for e in leaves(eg):
            self.assertNotEqual(match and e in match_leaves, rest and e in rest_leaves)
        for part in [match, rest, sg]:
            if part is not None:
                self.assertEqual(eg.message, part.message)
                self.assertIs(eg.__cause__, part.__cause__)
                self.assertIs(eg.__context__, part.__context__)
                self.assertIs(eg.__traceback__, part.__traceback__)
                self.assertEqual(getattr(eg, '__notes__', None), getattr(part, '__notes__', None))

        def tbs_for_leaf(leaf, eg):
            for e, tbs in leaf_generator(eg):
                if e is leaf:
                    return tbs

        def tb_linenos(tbs):
            return [tb.tb_lineno for tb in tbs if tb]
        for part in [match, rest, sg]:
            for e in leaves(part):
                self.assertSequenceEqual(tb_linenos(tbs_for_leaf(e, eg)), tb_linenos(tbs_for_leaf(e, part)))
        return (match, rest)


# --- test body ---
self_eg = create_simple_eg()
self_eg_template = [ValueError(1), TypeError(int), ValueError(2)]
eg = self_eg

assert eg is eg.subgroup(BaseException)

assert eg is eg.subgroup(Exception)

assert eg is eg.subgroup(BaseExceptionGroup)

assert eg is eg.subgroup(ExceptionGroup)
print("ExceptionGroupSubgroupTests::test_basics_subgroup_by_type__passthrough: ok")
"###);
    assert_output(&out, r###"ExceptionGroupSubgroupTests::test_basics_subgroup_by_type__passthrough: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/exception_group/exception_group_subgroup_tests__test_basics_subgroup_split_bad_arg_type.py`.
#[test]
fn test_gen_behavior_core_exception_group_exception_group_subgroup_tests__test_basics_subgroup_split_bad_arg_type() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exception_group"
# dimension = "behavior"
# case = "exception_group_subgroup_tests__test_basics_subgroup_split_bad_arg_type"
# subject = "cpython.test_exception_group.ExceptionGroupSubgroupTests.test_basics_subgroup_split__bad_arg_type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_exception_group.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_group.py::ExceptionGroupSubgroupTests::test_basics_subgroup_split__bad_arg_type
"""Auto-ported test: ExceptionGroupSubgroupTests::test_basics_subgroup_split__bad_arg_type (CPython 3.12 oracle)."""


import collections.abc
import types
import unittest
from test.support import C_RECURSION_LIMIT


def create_simple_eg():
    excs = []
    try:
        try:
            raise MemoryError('context and cause for ValueError(1)')
        except MemoryError as e:
            raise ValueError(1) from e
    except ValueError as e:
        excs.append(e)
    try:
        try:
            raise OSError('context for TypeError')
        except OSError as e:
            raise TypeError(int)
    except TypeError as e:
        excs.append(e)
    try:
        try:
            raise ImportError('context for ValueError(2)')
        except ImportError as e:
            raise ValueError(2)
    except ValueError as e:
        excs.append(e)
    try:
        raise ExceptionGroup('simple eg', excs)
    except ExceptionGroup as e:
        return e

class ExceptionGroupTestBase(unittest.TestCase):

    def assertMatchesTemplate(self, exc, exc_type, template):
        """ Assert that the exception matches the template

            A template describes the shape of exc. If exc is a
            leaf exception (i.e., not an exception group) then
            template is an exception instance that has the
            expected type and args value of exc. If exc is an
            exception group, then template is a list of the
            templates of its nested exceptions.
        """
        if exc_type is not None:
            self.assertIs(type(exc), exc_type)
        if isinstance(exc, BaseExceptionGroup):
            self.assertIsInstance(template, collections.abc.Sequence)
            self.assertEqual(len(exc.exceptions), len(template))
            for e, t in zip(exc.exceptions, template):
                self.assertMatchesTemplate(e, None, t)
        else:
            self.assertIsInstance(template, BaseException)
            self.assertEqual(type(exc), type(template))
            self.assertEqual(exc.args, template.args)

def leaf_generator(exc, tbs=None):
    if tbs is None:
        tbs = []
    tbs.append(exc.__traceback__)
    if isinstance(exc, BaseExceptionGroup):
        for e in exc.exceptions:
            yield from leaf_generator(e, tbs)
    else:
        yield (exc, tbs)
    tbs.pop()

def create_nested_eg():
    excs = []
    try:
        try:
            raise TypeError(bytes)
        except TypeError as e:
            raise ExceptionGroup('nested', [e])
    except ExceptionGroup as e:
        excs.append(e)
    try:
        try:
            raise MemoryError('out of memory')
        except MemoryError as e:
            raise ValueError(1) from e
    except ValueError as e:
        excs.append(e)
    try:
        raise ExceptionGroup('root', excs)
    except ExceptionGroup as eg:
        return eg

class ExceptionGroupSplitTestBase(ExceptionGroupTestBase):

    def split_exception_group(self, eg, types):
        """ Split an EG and do some sanity checks on the result """
        self.assertIsInstance(eg, BaseExceptionGroup)
        match, rest = eg.split(types)
        sg = eg.subgroup(types)
        if match is not None:
            self.assertIsInstance(match, BaseExceptionGroup)
            for e, _ in leaf_generator(match):
                self.assertIsInstance(e, types)
            self.assertIsNotNone(sg)
            self.assertIsInstance(sg, BaseExceptionGroup)
            for e, _ in leaf_generator(sg):
                self.assertIsInstance(e, types)
        if rest is not None:
            self.assertIsInstance(rest, BaseExceptionGroup)

        def leaves(exc):
            return [] if exc is None else [e for e, _ in leaf_generator(exc)]
        self.assertSequenceEqual(leaves(match), leaves(sg))
        match_leaves = leaves(match)
        rest_leaves = leaves(rest)
        self.assertEqual(len(leaves(eg)), len(leaves(match)) + len(leaves(rest)))
        for e in leaves(eg):
            self.assertNotEqual(match and e in match_leaves, rest and e in rest_leaves)
        for part in [match, rest, sg]:
            if part is not None:
                self.assertEqual(eg.message, part.message)
                self.assertIs(eg.__cause__, part.__cause__)
                self.assertIs(eg.__context__, part.__context__)
                self.assertIs(eg.__traceback__, part.__traceback__)
                self.assertEqual(getattr(eg, '__notes__', None), getattr(part, '__notes__', None))

        def tbs_for_leaf(leaf, eg):
            for e, tbs in leaf_generator(eg):
                if e is leaf:
                    return tbs

        def tb_linenos(tbs):
            return [tb.tb_lineno for tb in tbs if tb]
        for part in [match, rest, sg]:
            for e in leaves(part):
                self.assertSequenceEqual(tb_linenos(tbs_for_leaf(e, eg)), tb_linenos(tbs_for_leaf(e, part)))
        return (match, rest)


# --- test body ---
self_eg = create_simple_eg()
self_eg_template = [ValueError(1), TypeError(int), ValueError(2)]
bad_args = ['bad arg', OSError('instance not type'), [OSError, TypeError], (OSError, 42)]
for arg in bad_args:
    try:
        self_eg.subgroup(arg)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    try:
        self_eg.split(arg)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("ExceptionGroupSubgroupTests::test_basics_subgroup_split__bad_arg_type: ok")
"###);
    assert_output(&out, r###"ExceptionGroupSubgroupTests::test_basics_subgroup_split__bad_arg_type: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/exception_group/instance_creation__test_beg_and_specific_subclass_can_wrap_any_nonbase_exception.py`.
#[test]
fn test_gen_behavior_core_exception_group_instance_creation__test_beg_and_specific_subclass_can_wrap_any_nonbase_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exception_group"
# dimension = "behavior"
# case = "instance_creation__test_beg_and_specific_subclass_can_wrap_any_nonbase_exception"
# subject = "cpython.test_exception_group.InstanceCreation.test_BEG_and_specific_subclass_can_wrap_any_nonbase_exception"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_exception_group.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_group.py::InstanceCreation::test_BEG_and_specific_subclass_can_wrap_any_nonbase_exception
"""Auto-ported test: InstanceCreation::test_BEG_and_specific_subclass_can_wrap_any_nonbase_exception (CPython 3.12 oracle)."""


import collections.abc
import types
import unittest
from test.support import C_RECURSION_LIMIT


def create_simple_eg():
    excs = []
    try:
        try:
            raise MemoryError('context and cause for ValueError(1)')
        except MemoryError as e:
            raise ValueError(1) from e
    except ValueError as e:
        excs.append(e)
    try:
        try:
            raise OSError('context for TypeError')
        except OSError as e:
            raise TypeError(int)
    except TypeError as e:
        excs.append(e)
    try:
        try:
            raise ImportError('context for ValueError(2)')
        except ImportError as e:
            raise ValueError(2)
    except ValueError as e:
        excs.append(e)
    try:
        raise ExceptionGroup('simple eg', excs)
    except ExceptionGroup as e:
        return e

class ExceptionGroupTestBase(unittest.TestCase):

    def assertMatchesTemplate(self, exc, exc_type, template):
        """ Assert that the exception matches the template

            A template describes the shape of exc. If exc is a
            leaf exception (i.e., not an exception group) then
            template is an exception instance that has the
            expected type and args value of exc. If exc is an
            exception group, then template is a list of the
            templates of its nested exceptions.
        """
        if exc_type is not None:
            self.assertIs(type(exc), exc_type)
        if isinstance(exc, BaseExceptionGroup):
            self.assertIsInstance(template, collections.abc.Sequence)
            self.assertEqual(len(exc.exceptions), len(template))
            for e, t in zip(exc.exceptions, template):
                self.assertMatchesTemplate(e, None, t)
        else:
            self.assertIsInstance(template, BaseException)
            self.assertEqual(type(exc), type(template))
            self.assertEqual(exc.args, template.args)

def leaf_generator(exc, tbs=None):
    if tbs is None:
        tbs = []
    tbs.append(exc.__traceback__)
    if isinstance(exc, BaseExceptionGroup):
        for e in exc.exceptions:
            yield from leaf_generator(e, tbs)
    else:
        yield (exc, tbs)
    tbs.pop()

def create_nested_eg():
    excs = []
    try:
        try:
            raise TypeError(bytes)
        except TypeError as e:
            raise ExceptionGroup('nested', [e])
    except ExceptionGroup as e:
        excs.append(e)
    try:
        try:
            raise MemoryError('out of memory')
        except MemoryError as e:
            raise ValueError(1) from e
    except ValueError as e:
        excs.append(e)
    try:
        raise ExceptionGroup('root', excs)
    except ExceptionGroup as eg:
        return eg

class ExceptionGroupSplitTestBase(ExceptionGroupTestBase):

    def split_exception_group(self, eg, types):
        """ Split an EG and do some sanity checks on the result """
        self.assertIsInstance(eg, BaseExceptionGroup)
        match, rest = eg.split(types)
        sg = eg.subgroup(types)
        if match is not None:
            self.assertIsInstance(match, BaseExceptionGroup)
            for e, _ in leaf_generator(match):
                self.assertIsInstance(e, types)
            self.assertIsNotNone(sg)
            self.assertIsInstance(sg, BaseExceptionGroup)
            for e, _ in leaf_generator(sg):
                self.assertIsInstance(e, types)
        if rest is not None:
            self.assertIsInstance(rest, BaseExceptionGroup)

        def leaves(exc):
            return [] if exc is None else [e for e, _ in leaf_generator(exc)]
        self.assertSequenceEqual(leaves(match), leaves(sg))
        match_leaves = leaves(match)
        rest_leaves = leaves(rest)
        self.assertEqual(len(leaves(eg)), len(leaves(match)) + len(leaves(rest)))
        for e in leaves(eg):
            self.assertNotEqual(match and e in match_leaves, rest and e in rest_leaves)
        for part in [match, rest, sg]:
            if part is not None:
                self.assertEqual(eg.message, part.message)
                self.assertIs(eg.__cause__, part.__cause__)
                self.assertIs(eg.__context__, part.__context__)
                self.assertIs(eg.__traceback__, part.__traceback__)
                self.assertEqual(getattr(eg, '__notes__', None), getattr(part, '__notes__', None))

        def tbs_for_leaf(leaf, eg):
            for e, tbs in leaf_generator(eg):
                if e is leaf:
                    return tbs

        def tb_linenos(tbs):
            return [tb.tb_lineno for tb in tbs if tb]
        for part in [match, rest, sg]:
            for e in leaves(part):
                self.assertSequenceEqual(tb_linenos(tbs_for_leaf(e, eg)), tb_linenos(tbs_for_leaf(e, part)))
        return (match, rest)


# --- test body ---
class MyEG(BaseExceptionGroup, ValueError):
    pass
MyEG('eg', [ValueError(12), Exception()])
print("InstanceCreation::test_BEG_and_specific_subclass_can_wrap_any_nonbase_exception: ok")
"###);
    assert_output(&out, r###"InstanceCreation::test_BEG_and_specific_subclass_can_wrap_any_nonbase_exception: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/exception_group/instance_creation__test_eg_and_specific_subclass_can_wrap_any_nonbase_exception.py`.
#[test]
fn test_gen_behavior_core_exception_group_instance_creation__test_eg_and_specific_subclass_can_wrap_any_nonbase_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exception_group"
# dimension = "behavior"
# case = "instance_creation__test_eg_and_specific_subclass_can_wrap_any_nonbase_exception"
# subject = "cpython.test_exception_group.InstanceCreation.test_EG_and_specific_subclass_can_wrap_any_nonbase_exception"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_exception_group.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_group.py::InstanceCreation::test_EG_and_specific_subclass_can_wrap_any_nonbase_exception
"""Auto-ported test: InstanceCreation::test_EG_and_specific_subclass_can_wrap_any_nonbase_exception (CPython 3.12 oracle)."""


import collections.abc
import types
import unittest
from test.support import C_RECURSION_LIMIT


def create_simple_eg():
    excs = []
    try:
        try:
            raise MemoryError('context and cause for ValueError(1)')
        except MemoryError as e:
            raise ValueError(1) from e
    except ValueError as e:
        excs.append(e)
    try:
        try:
            raise OSError('context for TypeError')
        except OSError as e:
            raise TypeError(int)
    except TypeError as e:
        excs.append(e)
    try:
        try:
            raise ImportError('context for ValueError(2)')
        except ImportError as e:
            raise ValueError(2)
    except ValueError as e:
        excs.append(e)
    try:
        raise ExceptionGroup('simple eg', excs)
    except ExceptionGroup as e:
        return e

class ExceptionGroupTestBase(unittest.TestCase):

    def assertMatchesTemplate(self, exc, exc_type, template):
        """ Assert that the exception matches the template

            A template describes the shape of exc. If exc is a
            leaf exception (i.e., not an exception group) then
            template is an exception instance that has the
            expected type and args value of exc. If exc is an
            exception group, then template is a list of the
            templates of its nested exceptions.
        """
        if exc_type is not None:
            self.assertIs(type(exc), exc_type)
        if isinstance(exc, BaseExceptionGroup):
            self.assertIsInstance(template, collections.abc.Sequence)
            self.assertEqual(len(exc.exceptions), len(template))
            for e, t in zip(exc.exceptions, template):
                self.assertMatchesTemplate(e, None, t)
        else:
            self.assertIsInstance(template, BaseException)
            self.assertEqual(type(exc), type(template))
            self.assertEqual(exc.args, template.args)

def leaf_generator(exc, tbs=None):
    if tbs is None:
        tbs = []
    tbs.append(exc.__traceback__)
    if isinstance(exc, BaseExceptionGroup):
        for e in exc.exceptions:
            yield from leaf_generator(e, tbs)
    else:
        yield (exc, tbs)
    tbs.pop()

def create_nested_eg():
    excs = []
    try:
        try:
            raise TypeError(bytes)
        except TypeError as e:
            raise ExceptionGroup('nested', [e])
    except ExceptionGroup as e:
        excs.append(e)
    try:
        try:
            raise MemoryError('out of memory')
        except MemoryError as e:
            raise ValueError(1) from e
    except ValueError as e:
        excs.append(e)
    try:
        raise ExceptionGroup('root', excs)
    except ExceptionGroup as eg:
        return eg

class ExceptionGroupSplitTestBase(ExceptionGroupTestBase):

    def split_exception_group(self, eg, types):
        """ Split an EG and do some sanity checks on the result """
        self.assertIsInstance(eg, BaseExceptionGroup)
        match, rest = eg.split(types)
        sg = eg.subgroup(types)
        if match is not None:
            self.assertIsInstance(match, BaseExceptionGroup)
            for e, _ in leaf_generator(match):
                self.assertIsInstance(e, types)
            self.assertIsNotNone(sg)
            self.assertIsInstance(sg, BaseExceptionGroup)
            for e, _ in leaf_generator(sg):
                self.assertIsInstance(e, types)
        if rest is not None:
            self.assertIsInstance(rest, BaseExceptionGroup)

        def leaves(exc):
            return [] if exc is None else [e for e, _ in leaf_generator(exc)]
        self.assertSequenceEqual(leaves(match), leaves(sg))
        match_leaves = leaves(match)
        rest_leaves = leaves(rest)
        self.assertEqual(len(leaves(eg)), len(leaves(match)) + len(leaves(rest)))
        for e in leaves(eg):
            self.assertNotEqual(match and e in match_leaves, rest and e in rest_leaves)
        for part in [match, rest, sg]:
            if part is not None:
                self.assertEqual(eg.message, part.message)
                self.assertIs(eg.__cause__, part.__cause__)
                self.assertIs(eg.__context__, part.__context__)
                self.assertIs(eg.__traceback__, part.__traceback__)
                self.assertEqual(getattr(eg, '__notes__', None), getattr(part, '__notes__', None))

        def tbs_for_leaf(leaf, eg):
            for e, tbs in leaf_generator(eg):
                if e is leaf:
                    return tbs

        def tb_linenos(tbs):
            return [tb.tb_lineno for tb in tbs if tb]
        for part in [match, rest, sg]:
            for e in leaves(part):
                self.assertSequenceEqual(tb_linenos(tbs_for_leaf(e, eg)), tb_linenos(tbs_for_leaf(e, part)))
        return (match, rest)


# --- test body ---
class MyEG(ExceptionGroup, ValueError):
    pass
MyEG('eg', [ValueError(12), Exception()])
print("InstanceCreation::test_EG_and_specific_subclass_can_wrap_any_nonbase_exception: ok")
"###);
    assert_output(&out, r###"InstanceCreation::test_EG_and_specific_subclass_can_wrap_any_nonbase_exception: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/exception_group/nested_exception_group_split_test__test_split_does_not_copy_non_sequence_notes.py`.
#[test]
fn test_gen_behavior_core_exception_group_nested_exception_group_split_test__test_split_does_not_copy_non_sequence_notes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exception_group"
# dimension = "behavior"
# case = "nested_exception_group_split_test__test_split_does_not_copy_non_sequence_notes"
# subject = "cpython.test_exception_group.NestedExceptionGroupSplitTest.test_split_does_not_copy_non_sequence_notes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_exception_group.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_group.py::NestedExceptionGroupSplitTest::test_split_does_not_copy_non_sequence_notes
"""Auto-ported test: NestedExceptionGroupSplitTest::test_split_does_not_copy_non_sequence_notes (CPython 3.12 oracle)."""


import collections.abc
import types
import unittest
from test.support import C_RECURSION_LIMIT


def create_simple_eg():
    excs = []
    try:
        try:
            raise MemoryError('context and cause for ValueError(1)')
        except MemoryError as e:
            raise ValueError(1) from e
    except ValueError as e:
        excs.append(e)
    try:
        try:
            raise OSError('context for TypeError')
        except OSError as e:
            raise TypeError(int)
    except TypeError as e:
        excs.append(e)
    try:
        try:
            raise ImportError('context for ValueError(2)')
        except ImportError as e:
            raise ValueError(2)
    except ValueError as e:
        excs.append(e)
    try:
        raise ExceptionGroup('simple eg', excs)
    except ExceptionGroup as e:
        return e

class ExceptionGroupTestBase(unittest.TestCase):

    def assertMatchesTemplate(self, exc, exc_type, template):
        """ Assert that the exception matches the template

            A template describes the shape of exc. If exc is a
            leaf exception (i.e., not an exception group) then
            template is an exception instance that has the
            expected type and args value of exc. If exc is an
            exception group, then template is a list of the
            templates of its nested exceptions.
        """
        if exc_type is not None:
            self.assertIs(type(exc), exc_type)
        if isinstance(exc, BaseExceptionGroup):
            self.assertIsInstance(template, collections.abc.Sequence)
            self.assertEqual(len(exc.exceptions), len(template))
            for e, t in zip(exc.exceptions, template):
                self.assertMatchesTemplate(e, None, t)
        else:
            self.assertIsInstance(template, BaseException)
            self.assertEqual(type(exc), type(template))
            self.assertEqual(exc.args, template.args)

def leaf_generator(exc, tbs=None):
    if tbs is None:
        tbs = []
    tbs.append(exc.__traceback__)
    if isinstance(exc, BaseExceptionGroup):
        for e in exc.exceptions:
            yield from leaf_generator(e, tbs)
    else:
        yield (exc, tbs)
    tbs.pop()

def create_nested_eg():
    excs = []
    try:
        try:
            raise TypeError(bytes)
        except TypeError as e:
            raise ExceptionGroup('nested', [e])
    except ExceptionGroup as e:
        excs.append(e)
    try:
        try:
            raise MemoryError('out of memory')
        except MemoryError as e:
            raise ValueError(1) from e
    except ValueError as e:
        excs.append(e)
    try:
        raise ExceptionGroup('root', excs)
    except ExceptionGroup as eg:
        return eg

class ExceptionGroupSplitTestBase(ExceptionGroupTestBase):

    def split_exception_group(self, eg, types):
        """ Split an EG and do some sanity checks on the result """
        self.assertIsInstance(eg, BaseExceptionGroup)
        match, rest = eg.split(types)
        sg = eg.subgroup(types)
        if match is not None:
            self.assertIsInstance(match, BaseExceptionGroup)
            for e, _ in leaf_generator(match):
                self.assertIsInstance(e, types)
            self.assertIsNotNone(sg)
            self.assertIsInstance(sg, BaseExceptionGroup)
            for e, _ in leaf_generator(sg):
                self.assertIsInstance(e, types)
        if rest is not None:
            self.assertIsInstance(rest, BaseExceptionGroup)

        def leaves(exc):
            return [] if exc is None else [e for e, _ in leaf_generator(exc)]
        self.assertSequenceEqual(leaves(match), leaves(sg))
        match_leaves = leaves(match)
        rest_leaves = leaves(rest)
        self.assertEqual(len(leaves(eg)), len(leaves(match)) + len(leaves(rest)))
        for e in leaves(eg):
            self.assertNotEqual(match and e in match_leaves, rest and e in rest_leaves)
        for part in [match, rest, sg]:
            if part is not None:
                self.assertEqual(eg.message, part.message)
                self.assertIs(eg.__cause__, part.__cause__)
                self.assertIs(eg.__context__, part.__context__)
                self.assertIs(eg.__traceback__, part.__traceback__)
                self.assertEqual(getattr(eg, '__notes__', None), getattr(part, '__notes__', None))

        def tbs_for_leaf(leaf, eg):
            for e, tbs in leaf_generator(eg):
                if e is leaf:
                    return tbs

        def tb_linenos(tbs):
            return [tb.tb_lineno for tb in tbs if tb]
        for part in [match, rest, sg]:
            for e in leaves(part):
                self.assertSequenceEqual(tb_linenos(tbs_for_leaf(e, eg)), tb_linenos(tbs_for_leaf(e, part)))
        return (match, rest)


# --- test body ---
eg = ExceptionGroup('eg', [ValueError(1), TypeError(2)])
eg.__notes__ = 123
match, rest = eg.split(TypeError)

assert not hasattr(match, '__notes__')

assert not hasattr(rest, '__notes__')
print("NestedExceptionGroupSplitTest::test_split_does_not_copy_non_sequence_notes: ok")
"###);
    assert_output(&out, r###"NestedExceptionGroupSplitTest::test_split_does_not_copy_non_sequence_notes: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/exception_group/test_exception_group_type_hierarchy__test_exception_group_types.py`.
#[test]
fn test_gen_behavior_core_exception_group_test_exception_group_type_hierarchy__test_exception_group_types() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exception_group"
# dimension = "behavior"
# case = "test_exception_group_type_hierarchy__test_exception_group_types"
# subject = "cpython.test_exception_group.TestExceptionGroupTypeHierarchy.test_exception_group_types"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_exception_group.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_group.py::TestExceptionGroupTypeHierarchy::test_exception_group_types
"""Auto-ported test: TestExceptionGroupTypeHierarchy::test_exception_group_types (CPython 3.12 oracle)."""


import collections.abc
import types
import unittest
from test.support import C_RECURSION_LIMIT


def create_simple_eg():
    excs = []
    try:
        try:
            raise MemoryError('context and cause for ValueError(1)')
        except MemoryError as e:
            raise ValueError(1) from e
    except ValueError as e:
        excs.append(e)
    try:
        try:
            raise OSError('context for TypeError')
        except OSError as e:
            raise TypeError(int)
    except TypeError as e:
        excs.append(e)
    try:
        try:
            raise ImportError('context for ValueError(2)')
        except ImportError as e:
            raise ValueError(2)
    except ValueError as e:
        excs.append(e)
    try:
        raise ExceptionGroup('simple eg', excs)
    except ExceptionGroup as e:
        return e

class ExceptionGroupTestBase(unittest.TestCase):

    def assertMatchesTemplate(self, exc, exc_type, template):
        """ Assert that the exception matches the template

            A template describes the shape of exc. If exc is a
            leaf exception (i.e., not an exception group) then
            template is an exception instance that has the
            expected type and args value of exc. If exc is an
            exception group, then template is a list of the
            templates of its nested exceptions.
        """
        if exc_type is not None:
            self.assertIs(type(exc), exc_type)
        if isinstance(exc, BaseExceptionGroup):
            self.assertIsInstance(template, collections.abc.Sequence)
            self.assertEqual(len(exc.exceptions), len(template))
            for e, t in zip(exc.exceptions, template):
                self.assertMatchesTemplate(e, None, t)
        else:
            self.assertIsInstance(template, BaseException)
            self.assertEqual(type(exc), type(template))
            self.assertEqual(exc.args, template.args)

def leaf_generator(exc, tbs=None):
    if tbs is None:
        tbs = []
    tbs.append(exc.__traceback__)
    if isinstance(exc, BaseExceptionGroup):
        for e in exc.exceptions:
            yield from leaf_generator(e, tbs)
    else:
        yield (exc, tbs)
    tbs.pop()

def create_nested_eg():
    excs = []
    try:
        try:
            raise TypeError(bytes)
        except TypeError as e:
            raise ExceptionGroup('nested', [e])
    except ExceptionGroup as e:
        excs.append(e)
    try:
        try:
            raise MemoryError('out of memory')
        except MemoryError as e:
            raise ValueError(1) from e
    except ValueError as e:
        excs.append(e)
    try:
        raise ExceptionGroup('root', excs)
    except ExceptionGroup as eg:
        return eg

class ExceptionGroupSplitTestBase(ExceptionGroupTestBase):

    def split_exception_group(self, eg, types):
        """ Split an EG and do some sanity checks on the result """
        self.assertIsInstance(eg, BaseExceptionGroup)
        match, rest = eg.split(types)
        sg = eg.subgroup(types)
        if match is not None:
            self.assertIsInstance(match, BaseExceptionGroup)
            for e, _ in leaf_generator(match):
                self.assertIsInstance(e, types)
            self.assertIsNotNone(sg)
            self.assertIsInstance(sg, BaseExceptionGroup)
            for e, _ in leaf_generator(sg):
                self.assertIsInstance(e, types)
        if rest is not None:
            self.assertIsInstance(rest, BaseExceptionGroup)

        def leaves(exc):
            return [] if exc is None else [e for e, _ in leaf_generator(exc)]
        self.assertSequenceEqual(leaves(match), leaves(sg))
        match_leaves = leaves(match)
        rest_leaves = leaves(rest)
        self.assertEqual(len(leaves(eg)), len(leaves(match)) + len(leaves(rest)))
        for e in leaves(eg):
            self.assertNotEqual(match and e in match_leaves, rest and e in rest_leaves)
        for part in [match, rest, sg]:
            if part is not None:
                self.assertEqual(eg.message, part.message)
                self.assertIs(eg.__cause__, part.__cause__)
                self.assertIs(eg.__context__, part.__context__)
                self.assertIs(eg.__traceback__, part.__traceback__)
                self.assertEqual(getattr(eg, '__notes__', None), getattr(part, '__notes__', None))

        def tbs_for_leaf(leaf, eg):
            for e, tbs in leaf_generator(eg):
                if e is leaf:
                    return tbs

        def tb_linenos(tbs):
            return [tb.tb_lineno for tb in tbs if tb]
        for part in [match, rest, sg]:
            for e in leaves(part):
                self.assertSequenceEqual(tb_linenos(tbs_for_leaf(e, eg)), tb_linenos(tbs_for_leaf(e, part)))
        return (match, rest)


# --- test body ---

assert issubclass(ExceptionGroup, Exception)

assert issubclass(ExceptionGroup, BaseExceptionGroup)

assert issubclass(BaseExceptionGroup, BaseException)
print("TestExceptionGroupTypeHierarchy::test_exception_group_types: ok")
"###);
    assert_output(&out, r###"TestExceptionGroupTypeHierarchy::test_exception_group_types: ok
"###);
}
