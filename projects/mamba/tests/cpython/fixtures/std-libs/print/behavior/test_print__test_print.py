# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "print"
# dimension = "behavior"
# case = "test_print__test_print"
# subject = "cpython.test_print.TestPrint.test_print"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_print.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_print.py::TestPrint::test_print
"""Auto-ported test: TestPrint::test_print (CPython 3.12 oracle)."""


from io import StringIO
from test import support


NotDefined = object()

dispatch = {
    (False, False, False): lambda args, sep, end, file: print(*args),
    (False, False, True): lambda args, sep, end, file: print(*args, file=file),
    (False, True, False): lambda args, sep, end, file: print(*args, end=end),
    (False, True, True): lambda args, sep, end, file: print(*args, end=end, file=file),
    (True, False, False): lambda args, sep, end, file: print(*args, sep=sep),
    (True, False, True): lambda args, sep, end, file: print(*args, sep=sep, file=file),
    (True, True, False): lambda args, sep, end, file: print(*args, sep=sep, end=end),
    (True, True, True): lambda args, sep, end, file: print(*args, sep=sep, end=end, file=file),
}


class ClassWith__str__:
    def __init__(self, x):
        self.x = x

    def __str__(self):
        return self.x


def check(expected, args, sep=NotDefined, end=NotDefined, file=NotDefined):
    fn = dispatch[(sep is not NotDefined, end is not NotDefined, file is not NotDefined)]

    with support.captured_stdout() as target:
        fn(args, sep, end, file)

    assert target.getvalue() == expected


def x(expected, args, sep=NotDefined, end=NotDefined):
    check(expected, args, sep=sep, end=end)

    output = StringIO()
    check("", args, sep=sep, end=end, file=output)
    assert output.getvalue() == expected


x("\n", ())
x("a\n", ("a",))
x("None\n", (None,))
x("1 2\n", (1, 2))
x("1   2\n", (1, " ", 2))
x("1*2\n", (1, 2), sep="*")
x("1 s", (1, "s"), end="")
x("a\nb\n", ("a", "b"), sep="\n")
x("1.01", (1.0, 1), sep="", end="")
x("1*a*1.3+", (1, "a", 1.3), sep="*", end="+")
x("a\n\nb\n", ("a\n", "b"), sep="\n")
x("\0+ +\0\n", ("\0", " ", "\0"), sep="+")

x("a\n b\n", ("a\n", "b"))
x("a\n b\n", ("a\n", "b"), sep=None)
x("a\n b\n", ("a\n", "b"), end=None)
x("a\n b\n", ("a\n", "b"), sep=None, end=None)

x("*\n", (ClassWith__str__("*"),))
x("abc 1\n", (ClassWith__str__("abc"), 1))

for kwargs in ({"sep": 3}, {"end": 3}):
    try:
        print("", **kwargs)
        raise AssertionError(f"expected TypeError for {kwargs!r}")
    except TypeError:
        pass

try:
    print("", file="")
    raise AssertionError("expected AttributeError for non-file output target")
except AttributeError:
    pass

print("TestPrint::test_print: ok")
