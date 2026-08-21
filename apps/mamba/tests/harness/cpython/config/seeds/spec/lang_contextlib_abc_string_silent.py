# Operational AssertionPass seed for SILENT divergences across the
# resource-management / abstract-base-class / string-template
# quartet pinned by atomic 160: `contextlib` (the documented
# `ExitStack` / `redirect_stdout` / `closing` class identifiers +
# `suppress` exception-swallow contract + `@contextmanager`
# decorator yield contract), `abc` (the documented
# `@abstractmethod` instantiation-prevention contract on
# `abc.ABC` subclasses), and `string` (the documented
# `string.printable` constant + `string.Template.substitute` /
# `safe_substitute` instance methods).
#
# The matching subset (contextlib.suppress / contextmanager /
# nullcontext hasattr surface, nullcontext value passthrough,
# copy.copy / deepcopy round-trip + mutation independence on
# nested lists + dict-of-list, abc.ABC / abstractmethod /
# ABCMeta hasattr surface, concrete subclass overriding
# @abstractmethod is instantiable, string.ascii_letters /
# ascii_lowercase / ascii_uppercase / digits / punctuation /
# hexdigits / octdigits constants, string.capwords) is covered
# by `test_contextlib_copy_abc_string_value_ops`; this fixture
# pins the CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • hasattr(contextlib, "ExitStack") is True — documented
#     reentrant cleanup-stack helper (mamba: False);
#   • hasattr(contextlib, "redirect_stdout") is True —
#     documented stdout-redirection context manager (mamba:
#     False);
#   • hasattr(contextlib, "closing") is True — documented
#     close()-on-exit wrapper (mamba: False);
#   • `with contextlib.suppress(ZeroDivisionError): 1/0` does
#     NOT raise — exception-swallow contract (mamba: raises
#     ZeroDivisionError, suppress is a no-op shim);
#   • `@contextlib.contextmanager` decorator preserves the
#     yielded value — `with cm() as v` binds v to 42 (mamba:
#     v is bound to 1, decorator drops the yielded value);
#   • abc.ABC subclass with `@abstractmethod` and no override
#     raises TypeError on instantiation (mamba: instantiation
#     succeeds, returns a working instance — abstract method
#     guard is a no-op);
#   • len(string.printable) == 100 — documented printable
#     character set (mamba: returns 0, string.printable is
#     empty);
#   • string.Template("Hello, $name!").substitute(name="alice")
#     == "Hello, alice!" — instance-method substitute contract
#     (mamba: AttributeError, 'dict' object has no attribute
#     'substitute' — Template is being constructed as a bare
#     dict);
#   • string.Template(...).safe_substitute(...) returns the
#     untouched template when keys are missing — non-raising
#     fallback contract (mamba: AttributeError, 'dict' object
#     has no attribute 'safe_substitute').
import contextlib as _contextlib_mod
import abc as _abc_mod
import string as _string_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / module-level helpers / instance methods
# that mamba's bundled type stubs do not surface accurately.
contextlib: Any = _contextlib_mod
abc: Any = _abc_mod
string: Any = _string_mod


# Abstract base class must live at module level — mamba elides
# class identifiers declared inside try/except blocks.
class _AbstractBase(abc.ABC):
    @abc.abstractmethod
    def foo(self) -> str: ...


_ledger: list[int] = []

# 1) contextlib — documented helper attribute surface
assert hasattr(contextlib, "ExitStack") == True; _ledger.append(1)
assert hasattr(contextlib, "redirect_stdout") == True; _ledger.append(1)
assert hasattr(contextlib, "closing") == True; _ledger.append(1)

# 2) contextlib.suppress — exception-swallow contract
_suppressed = False
try:
    with contextlib.suppress(ZeroDivisionError):
        _ = 1 / 0
    _suppressed = True
except ZeroDivisionError:
    _suppressed = False
assert _suppressed == True; _ledger.append(1)


# 3) @contextlib.contextmanager — preserves yielded value
@contextlib.contextmanager
def _cm():
    yield 42


with _cm() as _v:
    assert _v == 42; _ledger.append(1)

# 4) abc — @abstractmethod prevents instantiation
_raised = False
try:
    _bad = _AbstractBase()  # type: ignore[abstract]
    _raised = False
except TypeError:
    _raised = True
assert _raised == True; _ledger.append(1)

# 5) string.printable — len contract
assert len(string.printable) == 100; _ledger.append(1)

# 6) string.Template — substitute / safe_substitute contracts
_t = string.Template("Hello, $name!")
assert _t.substitute(name="alice") == "Hello, alice!"; _ledger.append(1)
assert _t.safe_substitute({"other": 1}) == "Hello, $name!"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_contextlib_abc_string_silent {sum(_ledger)} asserts")
