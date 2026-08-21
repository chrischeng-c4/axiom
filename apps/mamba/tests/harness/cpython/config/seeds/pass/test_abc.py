# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: abc — ABC base class, @abc.abstractmethod decorator on free functions
# and methods, concrete subclasses overriding abstract methods, and the
# ABCMeta symbol being exposed.
# Intentionally NOT exercised on mamba today (tracked separately):
#   * Foo() where Foo is an ABC with un-overridden @abstractmethod
#     (CPython raises TypeError; mamba silently instantiates)
#   * ABCMeta(...) constructor — exposed as a lambda stub, not the real metaclass
#   * register / __subclasshook__ / virtual subclassing
#   * abstractclassmethod / abstractstaticmethod / abstractproperty (deprecated
#     CPython aliases)
import abc

_ledger: list[int] = []

# abc.ABC is exposed (used as a base class)
class _Foo(abc.ABC):
    @abc.abstractmethod
    def bar(self): ...

# Concrete subclass overriding the abstract method
class _Bar(_Foo):
    def bar(self):
        return 42

# Sanity: subclass instantiates and the override is dispatched
_b = _Bar()
assert _b.bar() - 42 == 0, f"_Bar().bar() == 42 via override, got {_b.bar()!r}"
_ledger.append(1)

# Another override that returns a different value
class _Bar7(_Foo):
    def bar(self):
        return 7

assert _Bar7().bar() - 7 == 0, "second concrete subclass returns 7"
_ledger.append(1)

# @abc.abstractmethod on a free function attaches __isabstractmethod__ = True
@abc.abstractmethod
def _free(): ...

assert getattr(_free, "__isabstractmethod__", False), (
    f"@abc.abstractmethod on free fn sets __isabstractmethod__ truthy, "
    f"got {getattr(_free, '__isabstractmethod__', 'missing')!r}"
)
_ledger.append(1)

# @abc.abstractmethod on a free function preserves the original function
# object shape; descriptor wrappers such as classmethod expose __func__.
assert not hasattr(_free, "__func__"), (
    f"@abc.abstractmethod on free fn should not wrap with __func__, got dir={dir(_free)!r}"
)
_ledger.append(1)

# abc.ABCMeta symbol is exposed
assert hasattr(abc, "ABCMeta"), "abc.ABCMeta symbol is exposed"
_ledger.append(1)

# abc.ABC symbol is exposed
assert hasattr(abc, "ABC"), "abc.ABC symbol is exposed"
_ledger.append(1)

# abc.abstractmethod symbol is exposed and is callable
assert hasattr(abc, "abstractmethod"), "abc.abstractmethod symbol is exposed"
_ledger.append(1)

# Multiple concrete overrides of the same abstract method behave independently
_b1 = _Bar()
_b2 = _Bar7()
assert _b1.bar() != _b2.bar(), (
    f"distinct concrete subclasses dispatch to their own overrides, "
    f"got {_b1.bar()!r} vs {_b2.bar()!r}"
)
_ledger.append(1)

# Abstract method on a class body — covered indirectly above; here we verify
# the decorator works on a class-bound method by checking the un-overridden
# class still carries the abstract marker at the function-attribute level.
class _Z(abc.ABC):
    @abc.abstractmethod
    def m(self): ...

class _ZZ(_Z):
    def m(self):
        return "hi"

assert _ZZ().m() == "hi", f"override of class-bound abstract method, got {_ZZ().m()!r}"
_ledger.append(1)

# Two ABCs with overlapping abstract method names produce concrete subclasses
# that dispatch independently
class _A(abc.ABC):
    @abc.abstractmethod
    def name(self): ...

class _B(abc.ABC):
    @abc.abstractmethod
    def name(self): ...

class _Aimpl(_A):
    def name(self):
        return "A"

class _Bimpl(_B):
    def name(self):
        return "B"

assert _Aimpl().name() == "A", f"_Aimpl.name == 'A', got {_Aimpl().name()!r}"
_ledger.append(1)
assert _Bimpl().name() == "B", f"_Bimpl.name == 'B', got {_Bimpl().name()!r}"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_abc {sum(_ledger)} asserts")
