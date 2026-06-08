# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pdb"
# dimension = "behavior"
# case = "pdb_test_case__test_invalid_cmd_line_options"
# subject = "cpython.test_pdb.PdbTestCase.test_invalid_cmd_line_options"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pdb.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pdb.py::PdbTestCase::test_invalid_cmd_line_options
"""Auto-ported test: PdbTestCase::test_invalid_cmd_line_options (CPython 3.12 oracle)."""


import doctest
import os
import pdb
import sys
import types
import codecs
import unittest
import subprocess
import textwrap
import linecache
from contextlib import ExitStack, redirect_stdout
from io import StringIO
from test import support
from test.support import os_helper
from test.support.import_helper import import_module
from test.support.pty_helper import run_pty, FakeInput
from unittest.mock import patch


SKIP_ASYNCIO_TESTS = not support.has_socket_support

class PdbTestInput(object):
    """Context manager that makes testing Pdb in doctests easier."""

    def __init__(self, input):
        self.input = input

    def __enter__(self):
        self.real_stdin = sys.stdin
        sys.stdin = FakeInput(self.input)
        self.orig_trace = sys.gettrace() if hasattr(sys, 'gettrace') else None

    def __exit__(self, *exc):
        sys.stdin = self.real_stdin
        if self.orig_trace:
            sys.settrace(self.orig_trace)

def test_pdb_displayhook():
    """This tests the custom displayhook for pdb.

    >>> def test_function(foo, bar):
    ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    ...     pass

    >>> with PdbTestInput([
    ...     'foo',
    ...     'bar',
    ...     'for i in range(5): print(i)',
    ...     'continue',
    ... ]):
    ...     test_function(1, None)
    > <doctest test.test_pdb.test_pdb_displayhook[0]>(3)test_function()
    -> pass
    (Pdb) foo
    1
    (Pdb) bar
    (Pdb) for i in range(5): print(i)
    0
    1
    2
    3
    4
    (Pdb) continue
    """

def test_pdb_basic_commands():
    """Test the basic commands of pdb.

    >>> def test_function_2(foo, bar='default'):
    ...     print(foo)
    ...     for i in range(5):
    ...         print(i)
    ...     print(bar)
    ...     for i in range(10):
    ...         never_executed
    ...     print('after for')
    ...     print('...')
    ...     return foo.upper()

    >>> def test_function3(arg=None, *, kwonly=None):
    ...     pass

    >>> def test_function4(a, b, c, /):
    ...     pass

    >>> def test_function():
    ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    ...     ret = test_function_2('baz')
    ...     test_function3(kwonly=True)
    ...     test_function4(1, 2, 3)
    ...     print(ret)

    >>> with PdbTestInput([  # doctest: +ELLIPSIS, +NORMALIZE_WHITESPACE
    ...     'step',       # entering the function call
    ...     'args',       # display function args
    ...     'list',       # list function source
    ...     'bt',         # display backtrace
    ...     'up',         # step up to test_function()
    ...     'down',       # step down to test_function_2() again
    ...     'next',       # stepping to print(foo)
    ...     'next',       # stepping to the for loop
    ...     'step',       # stepping into the for loop
    ...     'until',      # continuing until out of the for loop
    ...     'next',       # executing the print(bar)
    ...     'jump 8',     # jump over second for loop
    ...     'return',     # return out of function
    ...     'retval',     # display return value
    ...     'next',       # step to test_function3()
    ...     'step',       # stepping into test_function3()
    ...     'args',       # display function args
    ...     'return',     # return out of function
    ...     'next',       # step to test_function4()
    ...     'step',       # stepping to test_function4()
    ...     'args',       # display function args
    ...     'continue',
    ... ]):
    ...    test_function()
    > <doctest test.test_pdb.test_pdb_basic_commands[3]>(3)test_function()
    -> ret = test_function_2('baz')
    (Pdb) step
    --Call--
    > <doctest test.test_pdb.test_pdb_basic_commands[0]>(1)test_function_2()
    -> def test_function_2(foo, bar='default'):
    (Pdb) args
    foo = 'baz'
    bar = 'default'
    (Pdb) list
      1  ->     def test_function_2(foo, bar='default'):
      2             print(foo)
      3             for i in range(5):
      4                 print(i)
      5             print(bar)
      6             for i in range(10):
      7                 never_executed
      8             print('after for')
      9             print('...')
     10             return foo.upper()
    [EOF]
    (Pdb) bt
    ...
      <doctest test.test_pdb.test_pdb_basic_commands[4]>(25)<module>()
    -> test_function()
      <doctest test.test_pdb.test_pdb_basic_commands[3]>(3)test_function()
    -> ret = test_function_2('baz')
    > <doctest test.test_pdb.test_pdb_basic_commands[0]>(1)test_function_2()
    -> def test_function_2(foo, bar='default'):
    (Pdb) up
    > <doctest test.test_pdb.test_pdb_basic_commands[3]>(3)test_function()
    -> ret = test_function_2('baz')
    (Pdb) down
    > <doctest test.test_pdb.test_pdb_basic_commands[0]>(1)test_function_2()
    -> def test_function_2(foo, bar='default'):
    (Pdb) next
    > <doctest test.test_pdb.test_pdb_basic_commands[0]>(2)test_function_2()
    -> print(foo)
    (Pdb) next
    baz
    > <doctest test.test_pdb.test_pdb_basic_commands[0]>(3)test_function_2()
    -> for i in range(5):
    (Pdb) step
    > <doctest test.test_pdb.test_pdb_basic_commands[0]>(4)test_function_2()
    -> print(i)
    (Pdb) until
    0
    1
    2
    3
    4
    > <doctest test.test_pdb.test_pdb_basic_commands[0]>(5)test_function_2()
    -> print(bar)
    (Pdb) next
    default
    > <doctest test.test_pdb.test_pdb_basic_commands[0]>(6)test_function_2()
    -> for i in range(10):
    (Pdb) jump 8
    > <doctest test.test_pdb.test_pdb_basic_commands[0]>(8)test_function_2()
    -> print('after for')
    (Pdb) return
    after for
    ...
    --Return--
    > <doctest test.test_pdb.test_pdb_basic_commands[0]>(10)test_function_2()->'BAZ'
    -> return foo.upper()
    (Pdb) retval
    'BAZ'
    (Pdb) next
    > <doctest test.test_pdb.test_pdb_basic_commands[3]>(4)test_function()
    -> test_function3(kwonly=True)
    (Pdb) step
    --Call--
    > <doctest test.test_pdb.test_pdb_basic_commands[1]>(1)test_function3()
    -> def test_function3(arg=None, *, kwonly=None):
    (Pdb) args
    arg = None
    kwonly = True
    (Pdb) return
    --Return--
    > <doctest test.test_pdb.test_pdb_basic_commands[1]>(2)test_function3()->None
    -> pass
    (Pdb) next
    > <doctest test.test_pdb.test_pdb_basic_commands[3]>(5)test_function()
    -> test_function4(1, 2, 3)
    (Pdb) step
    --Call--
    > <doctest test.test_pdb.test_pdb_basic_commands[2]>(1)test_function4()
    -> def test_function4(a, b, c, /):
    (Pdb) args
    a = 1
    b = 2
    c = 3
    (Pdb) continue
    BAZ
    """

def reset_Breakpoint():
    import bdb
    bdb.Breakpoint.clearBreakpoints()

def test_pdb_breakpoint_commands():
    """Test basic commands related to breakpoints.

    >>> def test_function():
    ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    ...     print(1)
    ...     print(2)
    ...     print(3)
    ...     print(4)

    First, need to clear bdb state that might be left over from previous tests.
    Otherwise, the new breakpoints might get assigned different numbers.

    >>> reset_Breakpoint()

    Now test the breakpoint commands.  NORMALIZE_WHITESPACE is needed because
    the breakpoint list outputs a tab for the "stop only" and "ignore next"
    lines, which we don't want to put in here.

    >>> with PdbTestInput([  # doctest: +NORMALIZE_WHITESPACE
    ...     'break 3',
    ...     'break 4, +',
    ...     'disable 1',
    ...     'ignore 1 10',
    ...     'condition 1 1 < 2',
    ...     'condition 1 1 <',
    ...     'break 4',
    ...     'break 4',
    ...     'break',
    ...     'clear 3',
    ...     'break',
    ...     'condition 1',
    ...     'enable 1',
    ...     'clear 1',
    ...     'commands 2',
    ...     'p "42"',
    ...     'print("42", 7*6)',     # Issue 18764 (not about breakpoints)
    ...     'end',
    ...     'continue',  # will stop at breakpoint 2 (line 4)
    ...     'clear',     # clear all!
    ...     'y',
    ...     'tbreak 5',
    ...     'continue',  # will stop at temporary breakpoint
    ...     'break',     # make sure breakpoint is gone
    ...     'commands 10',  # out of range
    ...     'commands a',   # display help
    ...     'commands 4',   # already deleted
    ...     'break 6, undefined', # condition causing `NameError` during evaluation
    ...     'continue', # will stop, ignoring runtime error
    ...     'continue',
    ... ]):
    ...    test_function()
    > <doctest test.test_pdb.test_pdb_breakpoint_commands[0]>(3)test_function()
    -> print(1)
    (Pdb) break 3
    Breakpoint 1 at <doctest test.test_pdb.test_pdb_breakpoint_commands[0]>:3
    (Pdb) break 4, +
    *** Invalid condition +: SyntaxError: invalid syntax
    (Pdb) disable 1
    Disabled breakpoint 1 at <doctest test.test_pdb.test_pdb_breakpoint_commands[0]>:3
    (Pdb) ignore 1 10
    Will ignore next 10 crossings of breakpoint 1.
    (Pdb) condition 1 1 < 2
    New condition set for breakpoint 1.
    (Pdb) condition 1 1 <
    *** Invalid condition 1 <: SyntaxError: invalid syntax
    (Pdb) break 4
    Breakpoint 2 at <doctest test.test_pdb.test_pdb_breakpoint_commands[0]>:4
    (Pdb) break 4
    Breakpoint 3 at <doctest test.test_pdb.test_pdb_breakpoint_commands[0]>:4
    (Pdb) break
    Num Type         Disp Enb   Where
    1   breakpoint   keep no    at <doctest test.test_pdb.test_pdb_breakpoint_commands[0]>:3
            stop only if 1 < 2
            ignore next 10 hits
    2   breakpoint   keep yes   at <doctest test.test_pdb.test_pdb_breakpoint_commands[0]>:4
    3   breakpoint   keep yes   at <doctest test.test_pdb.test_pdb_breakpoint_commands[0]>:4
    (Pdb) clear 3
    Deleted breakpoint 3 at <doctest test.test_pdb.test_pdb_breakpoint_commands[0]>:4
    (Pdb) break
    Num Type         Disp Enb   Where
    1   breakpoint   keep no    at <doctest test.test_pdb.test_pdb_breakpoint_commands[0]>:3
            stop only if 1 < 2
            ignore next 10 hits
    2   breakpoint   keep yes   at <doctest test.test_pdb.test_pdb_breakpoint_commands[0]>:4
    (Pdb) condition 1
    Breakpoint 1 is now unconditional.
    (Pdb) enable 1
    Enabled breakpoint 1 at <doctest test.test_pdb.test_pdb_breakpoint_commands[0]>:3
    (Pdb) clear 1
    Deleted breakpoint 1 at <doctest test.test_pdb.test_pdb_breakpoint_commands[0]>:3
    (Pdb) commands 2
    (com) p "42"
    (com) print("42", 7*6)
    (com) end
    (Pdb) continue
    1
    '42'
    42 42
    > <doctest test.test_pdb.test_pdb_breakpoint_commands[0]>(4)test_function()
    -> print(2)
    (Pdb) clear
    Clear all breaks? y
    Deleted breakpoint 2 at <doctest test.test_pdb.test_pdb_breakpoint_commands[0]>:4
    (Pdb) tbreak 5
    Breakpoint 4 at <doctest test.test_pdb.test_pdb_breakpoint_commands[0]>:5
    (Pdb) continue
    2
    Deleted breakpoint 4 at <doctest test.test_pdb.test_pdb_breakpoint_commands[0]>:5
    > <doctest test.test_pdb.test_pdb_breakpoint_commands[0]>(5)test_function()
    -> print(3)
    (Pdb) break
    (Pdb) commands 10
    *** cannot set commands: Breakpoint number 10 out of range
    (Pdb) commands a
    *** Usage: commands [bnum]
            ...
            end
    (Pdb) commands 4
    *** cannot set commands: Breakpoint 4 already deleted
    (Pdb) break 6, undefined
    Breakpoint 5 at <doctest test.test_pdb.test_pdb_breakpoint_commands[0]>:6
    (Pdb) continue
    3
    > <doctest test.test_pdb.test_pdb_breakpoint_commands[0]>(6)test_function()
    -> print(4)
    (Pdb) continue
    4
    """

def test_pdb_breakpoint_on_annotated_function_def():
    """Test breakpoints on function definitions with annotation.

    >>> def foo[T]():
    ...     return 0

    >>> def bar() -> int:
    ...     return 0

    >>> def foobar[T]() -> int:
    ...     return 0

    >>> reset_Breakpoint()

    >>> def test_function():
    ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    ...     pass

    >>> with PdbTestInput([  # doctest: +NORMALIZE_WHITESPACE
    ...     'break foo',
    ...     'break bar',
    ...     'break foobar',
    ...     'continue',
    ... ]):
    ...    test_function()
    > <doctest test.test_pdb.test_pdb_breakpoint_on_annotated_function_def[4]>(3)test_function()
    -> pass
    (Pdb) break foo
    Breakpoint 1 at <doctest test.test_pdb.test_pdb_breakpoint_on_annotated_function_def[0]>:1
    (Pdb) break bar
    Breakpoint 2 at <doctest test.test_pdb.test_pdb_breakpoint_on_annotated_function_def[1]>:1
    (Pdb) break foobar
    Breakpoint 3 at <doctest test.test_pdb.test_pdb_breakpoint_on_annotated_function_def[2]>:1
    (Pdb) continue
    """

def test_pdb_breakpoints_preserved_across_interactive_sessions():
    """Breakpoints are remembered between interactive sessions

    >>> reset_Breakpoint()
    >>> with PdbTestInput([  # doctest: +ELLIPSIS, +NORMALIZE_WHITESPACE
    ...    'import test.test_pdb',
    ...    'break test.test_pdb.do_something',
    ...    'break test.test_pdb.do_nothing',
    ...    'break',
    ...    'continue',
    ... ]):
    ...    pdb.run('print()')
    > <string>(1)<module>()...
    (Pdb) import test.test_pdb
    (Pdb) break test.test_pdb.do_something
    Breakpoint 1 at ...test_pdb.py:...
    (Pdb) break test.test_pdb.do_nothing
    Breakpoint 2 at ...test_pdb.py:...
    (Pdb) break
    Num Type         Disp Enb   Where
    1   breakpoint   keep yes   at ...test_pdb.py:...
    2   breakpoint   keep yes   at ...test_pdb.py:...
    (Pdb) continue

    >>> with PdbTestInput([  # doctest: +ELLIPSIS, +NORMALIZE_WHITESPACE
    ...    'break',
    ...    'break pdb.find_function',
    ...    'break',
    ...    'clear 1',
    ...    'continue',
    ... ]):
    ...    pdb.run('print()')
    > <string>(1)<module>()...
    (Pdb) break
    Num Type         Disp Enb   Where
    1   breakpoint   keep yes   at ...test_pdb.py:...
    2   breakpoint   keep yes   at ...test_pdb.py:...
    (Pdb) break pdb.find_function
    Breakpoint 3 at ...pdb.py:...
    (Pdb) break
    Num Type         Disp Enb   Where
    1   breakpoint   keep yes   at ...test_pdb.py:...
    2   breakpoint   keep yes   at ...test_pdb.py:...
    3   breakpoint   keep yes   at ...pdb.py:...
    (Pdb) clear 1
    Deleted breakpoint 1 at ...test_pdb.py:...
    (Pdb) continue

    >>> with PdbTestInput([  # doctest: +ELLIPSIS, +NORMALIZE_WHITESPACE
    ...    'break',
    ...    'clear 2',
    ...    'clear 3',
    ...    'continue',
    ... ]):
    ...    pdb.run('print()')
    > <string>(1)<module>()...
    (Pdb) break
    Num Type         Disp Enb   Where
    2   breakpoint   keep yes   at ...test_pdb.py:...
    3   breakpoint   keep yes   at ...pdb.py:...
    (Pdb) clear 2
    Deleted breakpoint 2 at ...test_pdb.py:...
    (Pdb) clear 3
    Deleted breakpoint 3 at ...pdb.py:...
    (Pdb) continue
    """

def test_pdb_pp_repr_exc():
    """Test that do_p/do_pp do not swallow exceptions.

    >>> class BadRepr:
    ...     def __repr__(self):
    ...         raise Exception('repr_exc')
    >>> obj = BadRepr()

    >>> def test_function():
    ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()

    >>> with PdbTestInput([  # doctest: +NORMALIZE_WHITESPACE
    ...     'p obj',
    ...     'pp obj',
    ...     'continue',
    ... ]):
    ...    test_function()
    --Return--
    > <doctest test.test_pdb.test_pdb_pp_repr_exc[2]>(2)test_function()->None
    -> import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    (Pdb) p obj
    *** Exception: repr_exc
    (Pdb) pp obj
    *** Exception: repr_exc
    (Pdb) continue
    """

def test_pdb_empty_line():
    """Test that empty line repeats the last command.

    >>> def test_function():
    ...     x = 1
    ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    ...     pass
    ...     y = 2

    >>> with PdbTestInput([  # doctest: +NORMALIZE_WHITESPACE
    ...     'p x',
    ...     '',  # Should repeat p x
    ...     'n ;; p 0 ;; p x',  # Fill cmdqueue with multiple commands
    ...     '',  # Should still repeat p x
    ...     'continue',
    ... ]):
    ...    test_function()
    > <doctest test.test_pdb.test_pdb_empty_line[0]>(4)test_function()
    -> pass
    (Pdb) p x
    1
    (Pdb)
    1
    (Pdb) n ;; p 0 ;; p x
    0
    1
    > <doctest test.test_pdb.test_pdb_empty_line[0]>(5)test_function()
    -> y = 2
    (Pdb)
    1
    (Pdb) continue
    """

def do_nothing():
    pass

def do_something():
    print(42)

def test_list_commands():
    """Test the list and source commands of pdb.

    >>> def test_function_2(foo):
    ...     import test.test_pdb
    ...     test.test_pdb.do_nothing()
    ...     'some...'
    ...     'more...'
    ...     'code...'
    ...     'to...'
    ...     'make...'
    ...     'a...'
    ...     'long...'
    ...     'listing...'
    ...     'useful...'
    ...     '...'
    ...     '...'
    ...     return foo

    >>> def test_function():
    ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    ...     ret = test_function_2('baz')

    >>> with PdbTestInput([  # doctest: +ELLIPSIS, +NORMALIZE_WHITESPACE
    ...     'list',      # list first function
    ...     'step',      # step into second function
    ...     'list',      # list second function
    ...     'list',      # continue listing to EOF
    ...     'list 1,3',  # list specific lines
    ...     'list x',    # invalid argument
    ...     'next',      # step to import
    ...     'next',      # step over import
    ...     'step',      # step into do_nothing
    ...     'longlist',  # list all lines
    ...     'source do_something',  # list all lines of function
    ...     'source fooxxx',        # something that doesn't exit
    ...     'continue',
    ... ]):
    ...    test_function()
    > <doctest test.test_pdb.test_list_commands[1]>(3)test_function()
    -> ret = test_function_2('baz')
    (Pdb) list
      1         def test_function():
      2             import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
      3  ->         ret = test_function_2('baz')
    [EOF]
    (Pdb) step
    --Call--
    > <doctest test.test_pdb.test_list_commands[0]>(1)test_function_2()
    -> def test_function_2(foo):
    (Pdb) list
      1  ->     def test_function_2(foo):
      2             import test.test_pdb
      3             test.test_pdb.do_nothing()
      4             'some...'
      5             'more...'
      6             'code...'
      7             'to...'
      8             'make...'
      9             'a...'
     10             'long...'
     11             'listing...'
    (Pdb) list
     12             'useful...'
     13             '...'
     14             '...'
     15             return foo
    [EOF]
    (Pdb) list 1,3
      1  ->     def test_function_2(foo):
      2             import test.test_pdb
      3             test.test_pdb.do_nothing()
    (Pdb) list x
    *** ...
    (Pdb) next
    > <doctest test.test_pdb.test_list_commands[0]>(2)test_function_2()
    -> import test.test_pdb
    (Pdb) next
    > <doctest test.test_pdb.test_list_commands[0]>(3)test_function_2()
    -> test.test_pdb.do_nothing()
    (Pdb) step
    --Call--
    > ...test_pdb.py(...)do_nothing()
    -> def do_nothing():
    (Pdb) longlist
    ...  ->     def do_nothing():
    ...             pass
    (Pdb) source do_something
    ...         def do_something():
    ...             print(42)
    (Pdb) source fooxxx
    *** ...
    (Pdb) continue
    """

def test_pdb_whatis_command():
    """Test the whatis command

    >>> myvar = (1,2)
    >>> def myfunc():
    ...     pass

    >>> class MyClass:
    ...    def mymethod(self):
    ...        pass

    >>> def test_function():
    ...   import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()

    >>> with PdbTestInput([  # doctest: +ELLIPSIS, +NORMALIZE_WHITESPACE
    ...    'whatis myvar',
    ...    'whatis myfunc',
    ...    'whatis MyClass',
    ...    'whatis MyClass()',
    ...    'whatis MyClass.mymethod',
    ...    'whatis MyClass().mymethod',
    ...    'continue',
    ... ]):
    ...    test_function()
    --Return--
    > <doctest test.test_pdb.test_pdb_whatis_command[3]>(2)test_function()->None
    -> import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    (Pdb) whatis myvar
    <class 'tuple'>
    (Pdb) whatis myfunc
    Function myfunc
    (Pdb) whatis MyClass
    Class test.test_pdb.MyClass
    (Pdb) whatis MyClass()
    <class 'test.test_pdb.MyClass'>
    (Pdb) whatis MyClass.mymethod
    Function mymethod
    (Pdb) whatis MyClass().mymethod
    Method mymethod
    (Pdb) continue
    """

def test_pdb_display_command():
    """Test display command

    >>> def test_function():
    ...     a = 0
    ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    ...     a = 1
    ...     a = 2
    ...     a = 3
    ...     a = 4

    >>> with PdbTestInput([  # doctest: +ELLIPSIS
    ...     'display +',
    ...     'display',
    ...     'display a',
    ...     'n',
    ...     'display',
    ...     'undisplay a',
    ...     'n',
    ...     'display a',
    ...     'undisplay',
    ...     'display a < 1',
    ...     'n',
    ...     'display undefined',
    ...     'continue',
    ... ]):
    ...    test_function()
    > <doctest test.test_pdb.test_pdb_display_command[0]>(4)test_function()
    -> a = 1
    (Pdb) display +
    *** Unable to display +: SyntaxError: invalid syntax
    (Pdb) display
    No expression is being displayed
    (Pdb) display a
    display a: 0
    (Pdb) n
    > <doctest test.test_pdb.test_pdb_display_command[0]>(5)test_function()
    -> a = 2
    display a: 1  [old: 0]
    (Pdb) display
    Currently displaying:
    a: 1
    (Pdb) undisplay a
    (Pdb) n
    > <doctest test.test_pdb.test_pdb_display_command[0]>(6)test_function()
    -> a = 3
    (Pdb) display a
    display a: 2
    (Pdb) undisplay
    (Pdb) display a < 1
    display a < 1: False
    (Pdb) n
    > <doctest test.test_pdb.test_pdb_display_command[0]>(7)test_function()
    -> a = 4
    (Pdb) display undefined
    display undefined: ** raised NameError: name 'undefined' is not defined **
    (Pdb) continue
    """

def test_pdb_alias_command():
    """Test alias command

    >>> class A:
    ...     def __init__(self):
    ...         self.attr1 = 10
    ...         self.attr2 = 'str'
    ...     def method(self):
    ...         pass

    >>> def test_function():
    ...     o = A()
    ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    ...     o.method()

    >>> with PdbTestInput([  # doctest: +ELLIPSIS
    ...     'alias pi',
    ...     'alias pi for k in %1.__dict__.keys(): print(f"%1.{k} = {%1.__dict__[k]}")',
    ...     'alias ps pi self',
    ...     'alias ps',
    ...     'pi o',
    ...     's',
    ...     'ps',
    ...     'continue',
    ... ]):
    ...    test_function()
    > <doctest test.test_pdb.test_pdb_alias_command[1]>(4)test_function()
    -> o.method()
    (Pdb) alias pi
    *** Unknown alias 'pi'
    (Pdb) alias pi for k in %1.__dict__.keys(): print(f"%1.{k} = {%1.__dict__[k]}")
    (Pdb) alias ps pi self
    (Pdb) alias ps
    ps = pi self
    (Pdb) pi o
    o.attr1 = 10
    o.attr2 = str
    (Pdb) s
    --Call--
    > <doctest test.test_pdb.test_pdb_alias_command[0]>(5)method()
    -> def method(self):
    (Pdb) ps
    self.attr1 = 10
    self.attr2 = str
    (Pdb) continue
    """

def test_pdb_where_command():
    """Test where command

    >>> def g():
    ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()

    >>> def f():
    ...     g();

    >>> def test_function():
    ...     f()

    >>> with PdbTestInput([  # doctest: +ELLIPSIS
    ...     'w',
    ...     'where',
    ...     'u',
    ...     'w',
    ...     'continue',
    ... ]):
    ...    test_function()
    --Return--
    > <doctest test.test_pdb.test_pdb_where_command[0]>(2)g()->None
    -> import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    (Pdb) w
    ...
      <doctest test.test_pdb.test_pdb_where_command[3]>(8)<module>()
    -> test_function()
      <doctest test.test_pdb.test_pdb_where_command[2]>(2)test_function()
    -> f()
      <doctest test.test_pdb.test_pdb_where_command[1]>(2)f()
    -> g();
    > <doctest test.test_pdb.test_pdb_where_command[0]>(2)g()->None
    -> import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    (Pdb) where
    ...
      <doctest test.test_pdb.test_pdb_where_command[3]>(8)<module>()
    -> test_function()
      <doctest test.test_pdb.test_pdb_where_command[2]>(2)test_function()
    -> f()
      <doctest test.test_pdb.test_pdb_where_command[1]>(2)f()
    -> g();
    > <doctest test.test_pdb.test_pdb_where_command[0]>(2)g()->None
    -> import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    (Pdb) u
    > <doctest test.test_pdb.test_pdb_where_command[1]>(2)f()
    -> g();
    (Pdb) w
    ...
      <doctest test.test_pdb.test_pdb_where_command[3]>(8)<module>()
    -> test_function()
      <doctest test.test_pdb.test_pdb_where_command[2]>(2)test_function()
    -> f()
    > <doctest test.test_pdb.test_pdb_where_command[1]>(2)f()
    -> g();
      <doctest test.test_pdb.test_pdb_where_command[0]>(2)g()->None
    -> import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    (Pdb) continue
    """

def test_convenience_variables():
    """Test convenience variables

    >>> def util_function():
    ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    ...     try:
    ...         raise Exception('test')
    ...     except:
    ...         pass
    ...     return 1

    >>> def test_function():
    ...     util_function()

    >>> with PdbTestInput([  # doctest: +ELLIPSIS, +NORMALIZE_WHITESPACE
    ...     '$_frame.f_lineno', # Check frame convenience variable
    ...     '$ _frame',         # This should be a syntax error
    ...     '$a = 10',          # Set a convenience variable
    ...     '$a',               # Print its value
    ...     'p "$a"',           # Print the string $a
    ...     'p $a + 2',         # Do some calculation
    ...     'p f"$a = {$a}"',   # Make sure $ in string is not converted and f-string works
    ...     'u',                # Switch frame
    ...     '$_frame.f_lineno', # Make sure the frame changed
    ...     '$a',               # Make sure the value persists
    ...     'd',                # Go back to the original frame
    ...     'next',
    ...     '$a',               # The value should be gone
    ...     'next',
    ...     '$_exception',      # Check exception convenience variable
    ...     'next',
    ...     '$_exception',      # Exception should be gone
    ...     'return',
    ...     '$_retval',         # Check return convenience variable
    ...     'continue',
    ... ]):
    ...     test_function()
    > <doctest test.test_pdb.test_convenience_variables[0]>(3)util_function()
    -> try:
    (Pdb) $_frame.f_lineno
    3
    (Pdb) $ _frame
    *** SyntaxError: invalid syntax
    (Pdb) $a = 10
    (Pdb) $a
    10
    (Pdb) p "$a"
    '$a'
    (Pdb) p $a + 2
    12
    (Pdb) p f"$a = {$a}"
    '$a = 10'
    (Pdb) u
    > <doctest test.test_pdb.test_convenience_variables[1]>(2)test_function()
    -> util_function()
    (Pdb) $_frame.f_lineno
    2
    (Pdb) $a
    10
    (Pdb) d
    > <doctest test.test_pdb.test_convenience_variables[0]>(3)util_function()
    -> try:
    (Pdb) next
    > <doctest test.test_pdb.test_convenience_variables[0]>(4)util_function()
    -> raise Exception('test')
    (Pdb) $a
    *** KeyError: 'a'
    (Pdb) next
    Exception: test
    > <doctest test.test_pdb.test_convenience_variables[0]>(4)util_function()
    -> raise Exception('test')
    (Pdb) $_exception
    Exception('test')
    (Pdb) next
    > <doctest test.test_pdb.test_convenience_variables[0]>(5)util_function()
    -> except:
    (Pdb) $_exception
    *** KeyError: '_exception'
    (Pdb) return
    --Return--
    > <doctest test.test_pdb.test_convenience_variables[0]>(7)util_function()->1
    -> return 1
    (Pdb) $_retval
    1
    (Pdb) continue
    """

def test_post_mortem():
    """Test post mortem traceback debugging.

    >>> def test_function_2():
    ...     try:
    ...         1/0
    ...     finally:
    ...         print('Exception!')

    >>> def test_function():
    ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    ...     test_function_2()
    ...     print('Not reached.')

    >>> with PdbTestInput([  # doctest: +ELLIPSIS, +NORMALIZE_WHITESPACE
    ...     'next',      # step over exception-raising call
    ...     'bt',        # get a backtrace
    ...     'list',      # list code of test_function()
    ...     'down',      # step into test_function_2()
    ...     'list',      # list code of test_function_2()
    ...     'continue',
    ... ]):
    ...    try:
    ...        test_function()
    ...    except ZeroDivisionError:
    ...        print('Correctly reraised.')
    > <doctest test.test_pdb.test_post_mortem[1]>(3)test_function()
    -> test_function_2()
    (Pdb) next
    Exception!
    ZeroDivisionError: division by zero
    > <doctest test.test_pdb.test_post_mortem[1]>(3)test_function()
    -> test_function_2()
    (Pdb) bt
    ...
      <doctest test.test_pdb.test_post_mortem[2]>(10)<module>()
    -> test_function()
    > <doctest test.test_pdb.test_post_mortem[1]>(3)test_function()
    -> test_function_2()
      <doctest test.test_pdb.test_post_mortem[0]>(3)test_function_2()
    -> 1/0
    (Pdb) list
      1         def test_function():
      2             import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
      3  ->         test_function_2()
      4             print('Not reached.')
    [EOF]
    (Pdb) down
    > <doctest test.test_pdb.test_post_mortem[0]>(3)test_function_2()
    -> 1/0
    (Pdb) list
      1         def test_function_2():
      2             try:
      3  >>             1/0
      4             finally:
      5  ->             print('Exception!')
    [EOF]
    (Pdb) continue
    Correctly reraised.
    """

def test_pdb_return_to_different_file():
    """When pdb returns to a different file, it should not skip if f_trace is
       not already set

    >>> import pprint

    >>> class A:
    ...    def __repr__(self):
    ...        return 'A'

    >>> def test_function():
    ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    ...     pprint.pprint(A())

    >>> reset_Breakpoint()
    >>> with PdbTestInput([  # doctest: +ELLIPSIS, +NORMALIZE_WHITESPACE
    ...     'b A.__repr__',
    ...     'continue',
    ...     'return',
    ...     'next',
    ...     'return',
    ...     'return',
    ...     'continue',
    ... ]):
    ...    test_function()
    > <doctest test.test_pdb.test_pdb_return_to_different_file[2]>(3)test_function()
    -> pprint.pprint(A())
    (Pdb) b A.__repr__
    Breakpoint 1 at <doctest test.test_pdb.test_pdb_return_to_different_file[1]>:2
    (Pdb) continue
    > <doctest test.test_pdb.test_pdb_return_to_different_file[1]>(3)__repr__()
    -> return 'A'
    (Pdb) return
    --Return--
    > <doctest test.test_pdb.test_pdb_return_to_different_file[1]>(3)__repr__()->'A'
    -> return 'A'
    (Pdb) next
    > ...pprint.py..._safe_repr()
    -> return rep,...
    (Pdb) return
    --Return--
    > ...pprint.py..._safe_repr()->('A'...)
    -> return rep,...
    (Pdb) return
    --Return--
    > ...pprint.py...format()->('A'...)
    -> return...
    (Pdb) continue
    A
    """

def test_pdb_skip_modules():
    """This illustrates the simple case of module skipping.

    >>> def skip_module():
    ...     import string
    ...     import pdb; pdb.Pdb(skip=['stri*'], nosigint=True, readrc=False).set_trace()
    ...     string.capwords('FOO')

    >>> with PdbTestInput([
    ...     'step',
    ...     'continue',
    ... ]):
    ...     skip_module()
    > <doctest test.test_pdb.test_pdb_skip_modules[0]>(4)skip_module()
    -> string.capwords('FOO')
    (Pdb) step
    --Return--
    > <doctest test.test_pdb.test_pdb_skip_modules[0]>(4)skip_module()->None
    -> string.capwords('FOO')
    (Pdb) continue
    """

mod = types.ModuleType('module_to_skip')

exec('def foo_pony(callback): x = 1; callback(); return None', mod.__dict__)

def test_pdb_skip_modules_with_callback():
    """This illustrates skipping of modules that call into other code.

    >>> def skip_module():
    ...     def callback():
    ...         return None
    ...     import pdb; pdb.Pdb(skip=['module_to_skip*'], nosigint=True, readrc=False).set_trace()
    ...     mod.foo_pony(callback)

    >>> with PdbTestInput([
    ...     'step',
    ...     'step',
    ...     'step',
    ...     'step',
    ...     'step',
    ...     'continue',
    ... ]):
    ...     skip_module()
    ...     pass  # provides something to "step" to
    > <doctest test.test_pdb.test_pdb_skip_modules_with_callback[0]>(5)skip_module()
    -> mod.foo_pony(callback)
    (Pdb) step
    --Call--
    > <doctest test.test_pdb.test_pdb_skip_modules_with_callback[0]>(2)callback()
    -> def callback():
    (Pdb) step
    > <doctest test.test_pdb.test_pdb_skip_modules_with_callback[0]>(3)callback()
    -> return None
    (Pdb) step
    --Return--
    > <doctest test.test_pdb.test_pdb_skip_modules_with_callback[0]>(3)callback()->None
    -> return None
    (Pdb) step
    --Return--
    > <doctest test.test_pdb.test_pdb_skip_modules_with_callback[0]>(5)skip_module()->None
    -> mod.foo_pony(callback)
    (Pdb) step
    > <doctest test.test_pdb.test_pdb_skip_modules_with_callback[1]>(10)<module>()
    -> pass  # provides something to "step" to
    (Pdb) continue
    """

def test_pdb_continue_in_bottomframe():
    """Test that "continue" and "next" work properly in bottom frame (issue #5294).

    >>> def test_function():
    ...     import pdb, sys; inst = pdb.Pdb(nosigint=True, readrc=False)
    ...     inst.set_trace()
    ...     inst.botframe = sys._getframe()  # hackery to get the right botframe
    ...     print(1)
    ...     print(2)
    ...     print(3)
    ...     print(4)

    >>> with PdbTestInput([  # doctest: +ELLIPSIS
    ...     'next',
    ...     'break 7',
    ...     'continue',
    ...     'next',
    ...     'continue',
    ...     'continue',
    ... ]):
    ...    test_function()
    > <doctest test.test_pdb.test_pdb_continue_in_bottomframe[0]>(4)test_function()
    -> inst.botframe = sys._getframe()  # hackery to get the right botframe
    (Pdb) next
    > <doctest test.test_pdb.test_pdb_continue_in_bottomframe[0]>(5)test_function()
    -> print(1)
    (Pdb) break 7
    Breakpoint ... at <doctest test.test_pdb.test_pdb_continue_in_bottomframe[0]>:7
    (Pdb) continue
    1
    2
    > <doctest test.test_pdb.test_pdb_continue_in_bottomframe[0]>(7)test_function()
    -> print(3)
    (Pdb) next
    3
    > <doctest test.test_pdb.test_pdb_continue_in_bottomframe[0]>(8)test_function()
    -> print(4)
    (Pdb) continue
    4
    """

def pdb_invoke(method, arg):
    """Run pdb.method(arg)."""
    getattr(pdb.Pdb(nosigint=True, readrc=False), method)(arg)

def test_pdb_run_with_incorrect_argument():
    """Testing run and runeval with incorrect first argument.

    >>> pti = PdbTestInput(['continue',])
    >>> with pti:
    ...     pdb_invoke('run', lambda x: x)
    Traceback (most recent call last):
    TypeError: exec() arg 1 must be a string, bytes or code object

    >>> with pti:
    ...     pdb_invoke('runeval', lambda x: x)
    Traceback (most recent call last):
    TypeError: eval() arg 1 must be a string, bytes or code object
    """

def test_pdb_run_with_code_object():
    """Testing run and runeval with code object as a first argument.

    >>> with PdbTestInput(['step','x', 'continue']):  # doctest: +ELLIPSIS
    ...     pdb_invoke('run', compile('x=1', '<string>', 'exec'))
    > <string>(1)<module>()...
    (Pdb) step
    --Return--
    > <string>(1)<module>()->None
    (Pdb) x
    1
    (Pdb) continue

    >>> with PdbTestInput(['x', 'continue']):
    ...     x=0
    ...     pdb_invoke('runeval', compile('x+1', '<string>', 'eval'))
    > <string>(1)<module>()->None
    (Pdb) x
    1
    (Pdb) continue
    """

def test_next_until_return_at_return_event():
    """Test that pdb stops after a next/until/return issued at a return debug event.

    >>> def test_function_2():
    ...     x = 1
    ...     x = 2

    >>> def test_function():
    ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    ...     test_function_2()
    ...     test_function_2()
    ...     test_function_2()
    ...     end = 1

    >>> reset_Breakpoint()
    >>> with PdbTestInput(['break test_function_2',
    ...                    'continue',
    ...                    'return',
    ...                    'next',
    ...                    'continue',
    ...                    'return',
    ...                    'until',
    ...                    'continue',
    ...                    'return',
    ...                    'return',
    ...                    'continue']):
    ...     test_function()
    > <doctest test.test_pdb.test_next_until_return_at_return_event[1]>(3)test_function()
    -> test_function_2()
    (Pdb) break test_function_2
    Breakpoint 1 at <doctest test.test_pdb.test_next_until_return_at_return_event[0]>:1
    (Pdb) continue
    > <doctest test.test_pdb.test_next_until_return_at_return_event[0]>(2)test_function_2()
    -> x = 1
    (Pdb) return
    --Return--
    > <doctest test.test_pdb.test_next_until_return_at_return_event[0]>(3)test_function_2()->None
    -> x = 2
    (Pdb) next
    > <doctest test.test_pdb.test_next_until_return_at_return_event[1]>(4)test_function()
    -> test_function_2()
    (Pdb) continue
    > <doctest test.test_pdb.test_next_until_return_at_return_event[0]>(2)test_function_2()
    -> x = 1
    (Pdb) return
    --Return--
    > <doctest test.test_pdb.test_next_until_return_at_return_event[0]>(3)test_function_2()->None
    -> x = 2
    (Pdb) until
    > <doctest test.test_pdb.test_next_until_return_at_return_event[1]>(5)test_function()
    -> test_function_2()
    (Pdb) continue
    > <doctest test.test_pdb.test_next_until_return_at_return_event[0]>(2)test_function_2()
    -> x = 1
    (Pdb) return
    --Return--
    > <doctest test.test_pdb.test_next_until_return_at_return_event[0]>(3)test_function_2()->None
    -> x = 2
    (Pdb) return
    > <doctest test.test_pdb.test_next_until_return_at_return_event[1]>(6)test_function()
    -> end = 1
    (Pdb) continue
    """

def test_pdb_next_command_for_generator():
    """Testing skip unwindng stack on yield for generators for "next" command

    >>> def test_gen():
    ...     yield 0
    ...     return 1
    ...     yield 2

    >>> def test_function():
    ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    ...     it = test_gen()
    ...     try:
    ...         if next(it) != 0:
    ...             raise AssertionError
    ...         next(it)
    ...     except StopIteration as ex:
    ...         if ex.value != 1:
    ...             raise AssertionError
    ...     print("finished")

    >>> with PdbTestInput(['step',
    ...                    'step',
    ...                    'step',
    ...                    'next',
    ...                    'next',
    ...                    'step',
    ...                    'step',
    ...                    'continue']):
    ...     test_function()
    > <doctest test.test_pdb.test_pdb_next_command_for_generator[1]>(3)test_function()
    -> it = test_gen()
    (Pdb) step
    > <doctest test.test_pdb.test_pdb_next_command_for_generator[1]>(4)test_function()
    -> try:
    (Pdb) step
    > <doctest test.test_pdb.test_pdb_next_command_for_generator[1]>(5)test_function()
    -> if next(it) != 0:
    (Pdb) step
    --Call--
    > <doctest test.test_pdb.test_pdb_next_command_for_generator[0]>(1)test_gen()
    -> def test_gen():
    (Pdb) next
    > <doctest test.test_pdb.test_pdb_next_command_for_generator[0]>(2)test_gen()
    -> yield 0
    (Pdb) next
    > <doctest test.test_pdb.test_pdb_next_command_for_generator[0]>(3)test_gen()
    -> return 1
    (Pdb) step
    --Return--
    > <doctest test.test_pdb.test_pdb_next_command_for_generator[0]>(3)test_gen()->1
    -> return 1
    (Pdb) step
    StopIteration: 1
    > <doctest test.test_pdb.test_pdb_next_command_for_generator[1]>(7)test_function()
    -> next(it)
    (Pdb) continue
    finished
    """

if not SKIP_ASYNCIO_TESTS:

    def test_pdb_next_command_for_coroutine():
        """Testing skip unwindng stack on yield for coroutines for "next" command

        >>> import asyncio

        >>> async def test_coro():
        ...     await asyncio.sleep(0)
        ...     await asyncio.sleep(0)
        ...     await asyncio.sleep(0)

        >>> async def test_main():
        ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
        ...     await test_coro()

        >>> def test_function():
        ...     loop = asyncio.new_event_loop()
        ...     loop.run_until_complete(test_main())
        ...     loop.close()
        ...     asyncio.set_event_loop_policy(None)
        ...     print("finished")

        >>> with PdbTestInput(['step',
        ...                    'step',
        ...                    'next',
        ...                    'next',
        ...                    'next',
        ...                    'step',
        ...                    'continue']):
        ...     test_function()
        > <doctest test.test_pdb.test_pdb_next_command_for_coroutine[2]>(3)test_main()
        -> await test_coro()
        (Pdb) step
        --Call--
        > <doctest test.test_pdb.test_pdb_next_command_for_coroutine[1]>(1)test_coro()
        -> async def test_coro():
        (Pdb) step
        > <doctest test.test_pdb.test_pdb_next_command_for_coroutine[1]>(2)test_coro()
        -> await asyncio.sleep(0)
        (Pdb) next
        > <doctest test.test_pdb.test_pdb_next_command_for_coroutine[1]>(3)test_coro()
        -> await asyncio.sleep(0)
        (Pdb) next
        > <doctest test.test_pdb.test_pdb_next_command_for_coroutine[1]>(4)test_coro()
        -> await asyncio.sleep(0)
        (Pdb) next
        Internal StopIteration
        > <doctest test.test_pdb.test_pdb_next_command_for_coroutine[2]>(3)test_main()
        -> await test_coro()
        (Pdb) step
        --Return--
        > <doctest test.test_pdb.test_pdb_next_command_for_coroutine[2]>(3)test_main()->None
        -> await test_coro()
        (Pdb) continue
        finished
        """

    def test_pdb_next_command_for_asyncgen():
        """Testing skip unwindng stack on yield for coroutines for "next" command

        >>> import asyncio

        >>> async def agen():
        ...     yield 1
        ...     await asyncio.sleep(0)
        ...     yield 2

        >>> async def test_coro():
        ...     async for x in agen():
        ...         print(x)

        >>> async def test_main():
        ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
        ...     await test_coro()

        >>> def test_function():
        ...     loop = asyncio.new_event_loop()
        ...     loop.run_until_complete(test_main())
        ...     loop.close()
        ...     asyncio.set_event_loop_policy(None)
        ...     print("finished")

        >>> with PdbTestInput(['step',
        ...                    'step',
        ...                    'next',
        ...                    'next',
        ...                    'step',
        ...                    'next',
        ...                    'continue']):
        ...     test_function()
        > <doctest test.test_pdb.test_pdb_next_command_for_asyncgen[3]>(3)test_main()
        -> await test_coro()
        (Pdb) step
        --Call--
        > <doctest test.test_pdb.test_pdb_next_command_for_asyncgen[2]>(1)test_coro()
        -> async def test_coro():
        (Pdb) step
        > <doctest test.test_pdb.test_pdb_next_command_for_asyncgen[2]>(2)test_coro()
        -> async for x in agen():
        (Pdb) next
        > <doctest test.test_pdb.test_pdb_next_command_for_asyncgen[2]>(3)test_coro()
        -> print(x)
        (Pdb) next
        1
        > <doctest test.test_pdb.test_pdb_next_command_for_asyncgen[2]>(2)test_coro()
        -> async for x in agen():
        (Pdb) step
        --Call--
        > <doctest test.test_pdb.test_pdb_next_command_for_asyncgen[1]>(2)agen()
        -> yield 1
        (Pdb) next
        > <doctest test.test_pdb.test_pdb_next_command_for_asyncgen[1]>(3)agen()
        -> await asyncio.sleep(0)
        (Pdb) continue
        2
        finished
        """

def test_pdb_return_command_for_generator():
    """Testing no unwindng stack on yield for generators
       for "return" command

    >>> def test_gen():
    ...     yield 0
    ...     return 1
    ...     yield 2

    >>> def test_function():
    ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    ...     it = test_gen()
    ...     try:
    ...         if next(it) != 0:
    ...             raise AssertionError
    ...         next(it)
    ...     except StopIteration as ex:
    ...         if ex.value != 1:
    ...             raise AssertionError
    ...     print("finished")

    >>> with PdbTestInput(['step',
    ...                    'step',
    ...                    'step',
    ...                    'return',
    ...                    'step',
    ...                    'step',
    ...                    'continue']):
    ...     test_function()
    > <doctest test.test_pdb.test_pdb_return_command_for_generator[1]>(3)test_function()
    -> it = test_gen()
    (Pdb) step
    > <doctest test.test_pdb.test_pdb_return_command_for_generator[1]>(4)test_function()
    -> try:
    (Pdb) step
    > <doctest test.test_pdb.test_pdb_return_command_for_generator[1]>(5)test_function()
    -> if next(it) != 0:
    (Pdb) step
    --Call--
    > <doctest test.test_pdb.test_pdb_return_command_for_generator[0]>(1)test_gen()
    -> def test_gen():
    (Pdb) return
    StopIteration: 1
    > <doctest test.test_pdb.test_pdb_return_command_for_generator[1]>(7)test_function()
    -> next(it)
    (Pdb) step
    > <doctest test.test_pdb.test_pdb_return_command_for_generator[1]>(8)test_function()
    -> except StopIteration as ex:
    (Pdb) step
    > <doctest test.test_pdb.test_pdb_return_command_for_generator[1]>(9)test_function()
    -> if ex.value != 1:
    (Pdb) continue
    finished
    """

if not SKIP_ASYNCIO_TESTS:

    def test_pdb_return_command_for_coroutine():
        """Testing no unwindng stack on yield for coroutines for "return" command

        >>> import asyncio

        >>> async def test_coro():
        ...     await asyncio.sleep(0)
        ...     await asyncio.sleep(0)
        ...     await asyncio.sleep(0)

        >>> async def test_main():
        ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
        ...     await test_coro()

        >>> def test_function():
        ...     loop = asyncio.new_event_loop()
        ...     loop.run_until_complete(test_main())
        ...     loop.close()
        ...     asyncio.set_event_loop_policy(None)
        ...     print("finished")

        >>> with PdbTestInput(['step',
        ...                    'step',
        ...                    'next',
        ...                    'continue']):
        ...     test_function()
        > <doctest test.test_pdb.test_pdb_return_command_for_coroutine[2]>(3)test_main()
        -> await test_coro()
        (Pdb) step
        --Call--
        > <doctest test.test_pdb.test_pdb_return_command_for_coroutine[1]>(1)test_coro()
        -> async def test_coro():
        (Pdb) step
        > <doctest test.test_pdb.test_pdb_return_command_for_coroutine[1]>(2)test_coro()
        -> await asyncio.sleep(0)
        (Pdb) next
        > <doctest test.test_pdb.test_pdb_return_command_for_coroutine[1]>(3)test_coro()
        -> await asyncio.sleep(0)
        (Pdb) continue
        finished
        """

def test_pdb_until_command_for_generator():
    """Testing no unwindng stack on yield for generators
       for "until" command if target breakpoint is not reached

    >>> def test_gen():
    ...     yield 0
    ...     yield 1
    ...     yield 2

    >>> def test_function():
    ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    ...     for i in test_gen():
    ...         print(i)
    ...     print("finished")

    >>> with PdbTestInput(['step',
    ...                    'until 4',
    ...                    'step',
    ...                    'step',
    ...                    'continue']):
    ...     test_function()
    > <doctest test.test_pdb.test_pdb_until_command_for_generator[1]>(3)test_function()
    -> for i in test_gen():
    (Pdb) step
    --Call--
    > <doctest test.test_pdb.test_pdb_until_command_for_generator[0]>(1)test_gen()
    -> def test_gen():
    (Pdb) until 4
    0
    1
    > <doctest test.test_pdb.test_pdb_until_command_for_generator[0]>(4)test_gen()
    -> yield 2
    (Pdb) step
    --Return--
    > <doctest test.test_pdb.test_pdb_until_command_for_generator[0]>(4)test_gen()->2
    -> yield 2
    (Pdb) step
    > <doctest test.test_pdb.test_pdb_until_command_for_generator[1]>(4)test_function()
    -> print(i)
    (Pdb) continue
    2
    finished
    """

if not SKIP_ASYNCIO_TESTS:

    def test_pdb_until_command_for_coroutine():
        """Testing no unwindng stack for coroutines
        for "until" command if target breakpoint is not reached

        >>> import asyncio

        >>> async def test_coro():
        ...     print(0)
        ...     await asyncio.sleep(0)
        ...     print(1)
        ...     await asyncio.sleep(0)
        ...     print(2)
        ...     await asyncio.sleep(0)
        ...     print(3)

        >>> async def test_main():
        ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
        ...     await test_coro()

        >>> def test_function():
        ...     loop = asyncio.new_event_loop()
        ...     loop.run_until_complete(test_main())
        ...     loop.close()
        ...     asyncio.set_event_loop_policy(None)
        ...     print("finished")

        >>> with PdbTestInput(['step',
        ...                    'until 8',
        ...                    'continue']):
        ...     test_function()
        > <doctest test.test_pdb.test_pdb_until_command_for_coroutine[2]>(3)test_main()
        -> await test_coro()
        (Pdb) step
        --Call--
        > <doctest test.test_pdb.test_pdb_until_command_for_coroutine[1]>(1)test_coro()
        -> async def test_coro():
        (Pdb) until 8
        0
        1
        2
        > <doctest test.test_pdb.test_pdb_until_command_for_coroutine[1]>(8)test_coro()
        -> print(3)
        (Pdb) continue
        3
        finished
        """

def test_pdb_next_command_in_generator_for_loop():
    """The next command on returning from a generator controlled by a for loop.

    >>> def test_gen():
    ...     yield 0
    ...     return 1

    >>> def test_function():
    ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    ...     for i in test_gen():
    ...         print('value', i)
    ...     x = 123

    >>> reset_Breakpoint()
    >>> with PdbTestInput(['break test_gen',
    ...                    'continue',
    ...                    'next',
    ...                    'next',
    ...                    'next',
    ...                    'continue']):
    ...     test_function()
    > <doctest test.test_pdb.test_pdb_next_command_in_generator_for_loop[1]>(3)test_function()
    -> for i in test_gen():
    (Pdb) break test_gen
    Breakpoint 1 at <doctest test.test_pdb.test_pdb_next_command_in_generator_for_loop[0]>:1
    (Pdb) continue
    > <doctest test.test_pdb.test_pdb_next_command_in_generator_for_loop[0]>(2)test_gen()
    -> yield 0
    (Pdb) next
    value 0
    > <doctest test.test_pdb.test_pdb_next_command_in_generator_for_loop[0]>(3)test_gen()
    -> return 1
    (Pdb) next
    Internal StopIteration: 1
    > <doctest test.test_pdb.test_pdb_next_command_in_generator_for_loop[1]>(3)test_function()
    -> for i in test_gen():
    (Pdb) next
    > <doctest test.test_pdb.test_pdb_next_command_in_generator_for_loop[1]>(5)test_function()
    -> x = 123
    (Pdb) continue
    """

def test_pdb_next_command_subiterator():
    """The next command in a generator with a subiterator.

    >>> def test_subgenerator():
    ...     yield 0
    ...     return 1

    >>> def test_gen():
    ...     x = yield from test_subgenerator()
    ...     return x

    >>> def test_function():
    ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    ...     for i in test_gen():
    ...         print('value', i)
    ...     x = 123

    >>> with PdbTestInput(['step',
    ...                    'step',
    ...                    'next',
    ...                    'next',
    ...                    'next',
    ...                    'continue']):
    ...     test_function()
    > <doctest test.test_pdb.test_pdb_next_command_subiterator[2]>(3)test_function()
    -> for i in test_gen():
    (Pdb) step
    --Call--
    > <doctest test.test_pdb.test_pdb_next_command_subiterator[1]>(1)test_gen()
    -> def test_gen():
    (Pdb) step
    > <doctest test.test_pdb.test_pdb_next_command_subiterator[1]>(2)test_gen()
    -> x = yield from test_subgenerator()
    (Pdb) next
    value 0
    > <doctest test.test_pdb.test_pdb_next_command_subiterator[1]>(3)test_gen()
    -> return x
    (Pdb) next
    Internal StopIteration: 1
    > <doctest test.test_pdb.test_pdb_next_command_subiterator[2]>(3)test_function()
    -> for i in test_gen():
    (Pdb) next
    > <doctest test.test_pdb.test_pdb_next_command_subiterator[2]>(5)test_function()
    -> x = 123
    (Pdb) continue
    """

def test_pdb_issue_20766():
    """Test for reference leaks when the SIGINT handler is set.

    >>> def test_function():
    ...     i = 1
    ...     while i <= 2:
    ...         sess = pdb.Pdb()
    ...         sess.set_trace(sys._getframe())
    ...         print('pdb %d: %s' % (i, sess._previous_sigint_handler))
    ...         i += 1

    >>> reset_Breakpoint()
    >>> with PdbTestInput(['continue',
    ...                    'continue']):
    ...     test_function()
    > <doctest test.test_pdb.test_pdb_issue_20766[0]>(6)test_function()
    -> print('pdb %d: %s' % (i, sess._previous_sigint_handler))
    (Pdb) continue
    pdb 1: <built-in function default_int_handler>
    > <doctest test.test_pdb.test_pdb_issue_20766[0]>(6)test_function()
    -> print('pdb %d: %s' % (i, sess._previous_sigint_handler))
    (Pdb) continue
    pdb 2: <built-in function default_int_handler>
    """

def test_pdb_issue_43318():
    """echo breakpoints cleared with filename:lineno

    >>> def test_function():
    ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    ...     print(1)
    ...     print(2)
    ...     print(3)
    ...     print(4)
    >>> reset_Breakpoint()
    >>> with PdbTestInput([  # doctest: +NORMALIZE_WHITESPACE
    ...     'break 3',
    ...     'clear <doctest test.test_pdb.test_pdb_issue_43318[0]>:3',
    ...     'continue'
    ... ]):
    ...     test_function()
    > <doctest test.test_pdb.test_pdb_issue_43318[0]>(3)test_function()
    -> print(1)
    (Pdb) break 3
    Breakpoint 1 at <doctest test.test_pdb.test_pdb_issue_43318[0]>:3
    (Pdb) clear <doctest test.test_pdb.test_pdb_issue_43318[0]>:3
    Deleted breakpoint 1 at <doctest test.test_pdb.test_pdb_issue_43318[0]>:3
    (Pdb) continue
    1
    2
    3
    4
    """

def test_pdb_issue_gh_91742():
    """See GH-91742

    >>> def test_function():
    ...    __author__ = "pi"
    ...    __version__ = "3.14"
    ...
    ...    def about():
    ...        '''About'''
    ...        print(f"Author: {__author__!r}",
    ...            f"Version: {__version__!r}",
    ...            sep=" ")
    ...
    ...    import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    ...    about()


    >>> reset_Breakpoint()
    >>> with PdbTestInput([  # doctest: +NORMALIZE_WHITESPACE
    ...     'step',
    ...     'next',
    ...     'next',
    ...     'jump 5',
    ...     'continue'
    ... ]):
    ...     test_function()
    > <doctest test.test_pdb.test_pdb_issue_gh_91742[0]>(12)test_function()
    -> about()
    (Pdb) step
    --Call--
    > <doctest test.test_pdb.test_pdb_issue_gh_91742[0]>(5)about()
    -> def about():
    (Pdb) next
    > <doctest test.test_pdb.test_pdb_issue_gh_91742[0]>(7)about()
    -> print(f"Author: {__author__!r}",
    (Pdb) next
    > <doctest test.test_pdb.test_pdb_issue_gh_91742[0]>(8)about()
    -> f"Version: {__version__!r}",
    (Pdb) jump 5
    > <doctest test.test_pdb.test_pdb_issue_gh_91742[0]>(5)about()
    -> def about():
    (Pdb) continue
    Author: 'pi' Version: '3.14'
    """

def test_pdb_issue_gh_94215():
    """See GH-94215

    Check that frame_setlineno() does not leak references.

    >>> def test_function():
    ...    def func():
    ...        def inner(v): pass
    ...        inner(
    ...             42
    ...        )
    ...
    ...    import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    ...    func()

    >>> reset_Breakpoint()
    >>> with PdbTestInput([  # doctest: +NORMALIZE_WHITESPACE
    ...     'step',
    ...     'next',
    ...     'next',
    ...     'jump 3',
    ...     'next',
    ...     'next',
    ...     'jump 3',
    ...     'next',
    ...     'next',
    ...     'jump 3',
    ...     'continue'
    ... ]):
    ...     test_function()
    > <doctest test.test_pdb.test_pdb_issue_gh_94215[0]>(9)test_function()
    -> func()
    (Pdb) step
    --Call--
    > <doctest test.test_pdb.test_pdb_issue_gh_94215[0]>(2)func()
    -> def func():
    (Pdb) next
    > <doctest test.test_pdb.test_pdb_issue_gh_94215[0]>(3)func()
    -> def inner(v): pass
    (Pdb) next
    > <doctest test.test_pdb.test_pdb_issue_gh_94215[0]>(4)func()
    -> inner(
    (Pdb) jump 3
    > <doctest test.test_pdb.test_pdb_issue_gh_94215[0]>(3)func()
    -> def inner(v): pass
    (Pdb) next
    > <doctest test.test_pdb.test_pdb_issue_gh_94215[0]>(4)func()
    -> inner(
    (Pdb) next
    > <doctest test.test_pdb.test_pdb_issue_gh_94215[0]>(5)func()
    -> 42
    (Pdb) jump 3
    > <doctest test.test_pdb.test_pdb_issue_gh_94215[0]>(3)func()
    -> def inner(v): pass
    (Pdb) next
    > <doctest test.test_pdb.test_pdb_issue_gh_94215[0]>(4)func()
    -> inner(
    (Pdb) next
    > <doctest test.test_pdb.test_pdb_issue_gh_94215[0]>(5)func()
    -> 42
    (Pdb) jump 3
    > <doctest test.test_pdb.test_pdb_issue_gh_94215[0]>(3)func()
    -> def inner(v): pass
    (Pdb) continue
    """

def test_pdb_issue_gh_101673():
    """See GH-101673

    Make sure ll won't revert local variable assignment

    >>> def test_function():
    ...    a = 1
    ...    import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()

    >>> with PdbTestInput([  # doctest: +NORMALIZE_WHITESPACE
    ...     '!a = 2',
    ...     'll',
    ...     'p a',
    ...     'continue'
    ... ]):
    ...     test_function()
    --Return--
    > <doctest test.test_pdb.test_pdb_issue_gh_101673[0]>(3)test_function()->None
    -> import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    (Pdb) !a = 2
    (Pdb) ll
      1         def test_function():
      2            a = 1
      3  ->        import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    (Pdb) p a
    2
    (Pdb) continue
    """

def test_pdb_issue_gh_103225():
    """See GH-103225

    Make sure longlist uses 1-based line numbers in frames that correspond to a module

    >>> with PdbTestInput([  # doctest: +NORMALIZE_WHITESPACE
    ...     'longlist',
    ...     'continue'
    ... ]):
    ...     a = 1
    ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    ...     b = 2
    > <doctest test.test_pdb.test_pdb_issue_gh_103225[0]>(7)<module>()
    -> b = 2
    (Pdb) longlist
      1     with PdbTestInput([  # doctest: +NORMALIZE_WHITESPACE
      2         'longlist',
      3         'continue'
      4     ]):
      5         a = 1
      6         import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
      7  ->     b = 2
    (Pdb) continue
    """

def test_pdb_issue_gh_101517():
    """See GH-101517

    Make sure pdb doesn't crash when the exception is caught in a try/except* block

    >>> def test_function():
    ...     try:
    ...         raise KeyError
    ...     except* Exception as e:
    ...         import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()

    >>> with PdbTestInput([  # doctest: +NORMALIZE_WHITESPACE
    ...     'continue'
    ... ]):
    ...    test_function()
    --Return--
    > <doctest test.test_pdb.test_pdb_issue_gh_101517[0]>(None)test_function()->None
    -> Warning: lineno is None
    (Pdb) continue
    """

def test_pdb_issue_gh_108976():
    """See GH-108976
    Make sure setting f_trace_opcodes = True won't crash pdb
    >>> def test_function():
    ...     import sys
    ...     sys._getframe().f_trace_opcodes = True
    ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    ...     a = 1
    >>> with PdbTestInput([  # doctest: +NORMALIZE_WHITESPACE
    ...     'continue'
    ... ]):
    ...    test_function()
    bdb.Bdb.dispatch: unknown debugging event: 'opcode'
    > <doctest test.test_pdb.test_pdb_issue_gh_108976[0]>(5)test_function()
    -> a = 1
    (Pdb) continue
    """

def test_pdb_ambiguous_statements():
    """See GH-104301

    Make sure that ambiguous statements prefixed by '!' are properly disambiguated

    >>> with PdbTestInput([
    ...     '! n = 42',  # disambiguated statement: reassign the name n
    ...     'n',         # advance the debugger into the print()
    ...     'continue'
    ... ]):
    ...     n = -1
    ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    ...     print(f"The value of n is {n}")
    > <doctest test.test_pdb.test_pdb_ambiguous_statements[0]>(8)<module>()
    -> print(f"The value of n is {n}")
    (Pdb) ! n = 42
    (Pdb) n
    The value of n is 42
    > <doctest test.test_pdb.test_pdb_ambiguous_statements[0]>(1)<module>()
    -> with PdbTestInput([
    (Pdb) continue
    """

def test_pdb_frame_refleak():
    """
    pdb should not leak reference to frames

    >>> def frame_leaker(container):
    ...     import sys
    ...     container.append(sys._getframe())
    ...     import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    ...     pass

    >>> def test_function():
    ...     import gc
    ...     container = []
    ...     frame_leaker(container)  # c
    ...     print(len(gc.get_referrers(container[0])))
    ...     container = []
    ...     frame_leaker(container)  # n c
    ...     print(len(gc.get_referrers(container[0])))
    ...     container = []
    ...     frame_leaker(container)  # r c
    ...     print(len(gc.get_referrers(container[0])))

    >>> with PdbTestInput([  # doctest: +NORMALIZE_WHITESPACE
    ...     'continue',
    ...     'next',
    ...     'continue',
    ...     'return',
    ...     'continue',
    ... ]):
    ...    test_function()
    > <doctest test.test_pdb.test_pdb_frame_refleak[0]>(5)frame_leaker()
    -> pass
    (Pdb) continue
    1
    > <doctest test.test_pdb.test_pdb_frame_refleak[0]>(5)frame_leaker()
    -> pass
    (Pdb) next
    --Return--
    > <doctest test.test_pdb.test_pdb_frame_refleak[0]>(5)frame_leaker()->None
    -> pass
    (Pdb) continue
    1
    > <doctest test.test_pdb.test_pdb_frame_refleak[0]>(5)frame_leaker()
    -> pass
    (Pdb) return
    --Return--
    > <doctest test.test_pdb.test_pdb_frame_refleak[0]>(5)frame_leaker()->None
    -> pass
    (Pdb) continue
    1
    """

def test_pdb_issue_gh_65052():
    """See GH-65052

    args, retval and display should not crash if the object is not displayable
    >>> class A:
    ...     def __new__(cls):
    ...         import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    ...         return object.__new__(cls)
    ...     def __init__(self):
    ...         import pdb; pdb.Pdb(nosigint=True, readrc=False).set_trace()
    ...         self.a = 1
    ...     def __repr__(self):
    ...         return self.a

    >>> def test_function():
    ...     A()
    >>> with PdbTestInput([  # doctest: +ELLIPSIS +NORMALIZE_WHITESPACE
    ...     's',
    ...     'retval',
    ...     'continue',
    ...     'args',
    ...     'display self',
    ...     'display',
    ...     'continue',
    ... ]):
    ...    test_function()
    > <doctest test.test_pdb.test_pdb_issue_gh_65052[0]>(4)__new__()
    -> return object.__new__(cls)
    (Pdb) s
    --Return--
    > <doctest test.test_pdb.test_pdb_issue_gh_65052[0]>(4)__new__()-><A instance at ...>
    -> return object.__new__(cls)
    (Pdb) retval
    *** repr(retval) failed: AttributeError: 'A' object has no attribute 'a' ***
    (Pdb) continue
    > <doctest test.test_pdb.test_pdb_issue_gh_65052[0]>(7)__init__()
    -> self.a = 1
    (Pdb) args
    self = *** repr(self) failed: AttributeError: 'A' object has no attribute 'a' ***
    (Pdb) display self
    display self: *** repr(self) failed: AttributeError: 'A' object has no attribute 'a' ***
    (Pdb) display
    Currently displaying:
    self: *** repr(self) failed: AttributeError: 'A' object has no attribute 'a' ***
    (Pdb) continue
    """

def load_tests(loader, tests, pattern):
    from test import test_pdb
    tests.addTest(doctest.DocTestSuite(test_pdb))
    return tests


# --- test body ---
def _assert_find_function(file_content, func_name, expected):
    with open(os_helper.TESTFN, 'wb') as f:
        f.write(file_content)
    expected = None if not expected else (expected[0], os_helper.TESTFN, expected[1])

    assert expected == pdb.find_function(func_name, os_helper.TESTFN)

def _run_pdb(pdb_args, commands, expected_returncode=0):
    pass
    cmd = [sys.executable, '-m', 'pdb'] + pdb_args
    with subprocess.Popen(cmd, stdout=subprocess.PIPE, stdin=subprocess.PIPE, stderr=subprocess.STDOUT, env={**os.environ, 'PYTHONIOENCODING': 'utf-8'}) as proc:
        stdout, stderr = proc.communicate(str.encode(commands))
    stdout = stdout and bytes.decode(stdout)
    stderr = stderr and bytes.decode(stderr)

    assert proc.returncode == expected_returncode
    return (stdout, stderr)

def run_pdb_module(script, commands):
    """Runs the script code as part of a module"""
    self_module_name = 't_main'
    os_helper.rmtree(self_module_name)
    main_file = self_module_name + '/__main__.py'
    init_file = self_module_name + '/__init__.py'
    os.mkdir(self_module_name)
    with open(init_file, 'w') as f:
        pass
    with open(main_file, 'w') as f:
        f.write(textwrap.dedent(script))
    pass
    return _run_pdb(['-m', self_module_name], commands)

def run_pdb_script(script, commands, expected_returncode=0, pdbrc=None, remove_home=False):
    """Run 'script' lines with pdb and the pdb 'commands'."""
    filename = 'main.py'
    with open(filename, 'w') as f:
        f.write(textwrap.dedent(script))
    if pdbrc is not None:
        with open('.pdbrc', 'w') as f:
            f.write(textwrap.dedent(pdbrc))
        pass
    pass
    homesave = None
    if remove_home:
        homesave = os.environ.pop('HOME', None)
    try:
        stdout, stderr = _run_pdb([filename], commands, expected_returncode)
    finally:
        if homesave is not None:
            os.environ['HOME'] = homesave
    return (stdout, stderr)
stdout, stderr = _run_pdb(['-c'], '', expected_returncode=1)

assert f'Error: option -c requires argument' in stdout
stdout, stderr = _run_pdb(['--spam'], '', expected_returncode=1)

assert f'Error: option --spam not recognized' in stdout
print("PdbTestCase::test_invalid_cmd_line_options: ok")
