use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/abc/abcmeta_explicit_metaclass.py`.
#[test]
fn test_gen_behavior_std_libs_abc_abcmeta_explicit_metaclass() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "behavior"
# case = "abcmeta_explicit_metaclass"
# subject = "abc.ABCMeta"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""abc.ABCMeta: using ABCMeta directly as metaclass gives the same abstract-enforcement and isinstance behavior as inheriting ABC"""
import abc


class Animal(metaclass=abc.ABCMeta):
    @abc.abstractmethod
    def speak(self) -> str: ...


# Abstract enforcement applies just as with ABC inheritance.
_raised = False
try:
    Animal()
except TypeError:
    _raised = True
assert _raised, "ABCMeta-based abstract class is not instantiable"


class Dog(Animal):
    def speak(self) -> str:
        return "woof"


assert Dog().speak() == "woof", "concrete ABCMeta subclass works"
assert isinstance(Dog(), Animal), "Dog instance isinstance Animal"
assert issubclass(Dog, Animal), "Dog issubclass Animal"

print("abcmeta_explicit_metaclass OK")
"###);
    assert_output(&out, r###"abcmeta_explicit_metaclass OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/abc/abstract_base_not_instantiable.py`.
#[test]
fn test_gen_behavior_std_libs_abc_abstract_base_not_instantiable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "behavior"
# case = "abstract_base_not_instantiable"
# subject = "abc.ABC"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""abc.ABC: instantiating an ABC subclass with an unimplemented abstractmethod raises TypeError"""
import abc


class Base(abc.ABC):
    @abc.abstractmethod
    def do(self) -> int: ...


_raised = False
try:
    Base()
except TypeError:
    _raised = True
assert _raised, "abstract base raises TypeError on instantiation"

print("abstract_base_not_instantiable OK")
"###);
    assert_output(&out, r###"abstract_base_not_instantiable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/abc/abstractmethods_set_contents.py`.
#[test]
fn test_gen_behavior_std_libs_abc_abstractmethods_set_contents() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "behavior"
# case = "abstractmethods_set_contents"
# subject = "abc.ABC"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""abc.ABC: __abstractmethods__ lists exactly the unimplemented abstract names and is empty once all are overridden"""
import abc


class Shape(abc.ABC):
    @abc.abstractmethod
    def area(self) -> float: ...
    @abc.abstractmethod
    def perimeter(self) -> float: ...


# The base names both abstract methods.
assert "area" in Shape.__abstractmethods__, "area in __abstractmethods__"
assert "perimeter" in Shape.__abstractmethods__, "perimeter in __abstractmethods__"
assert set(Shape.__abstractmethods__) == {"area", "perimeter"}, "exact abstract set"


class Circle(Shape):
    def area(self) -> float:
        return 3.14
    def perimeter(self) -> float:
        return 6.28


# Overriding every abstract empties the set on the concrete subclass.
assert len(Circle.__abstractmethods__) == 0, "concrete subclass has empty __abstractmethods__"

print("abstractmethods_set_contents OK")
"###);
    assert_output(&out, r###"abstractmethods_set_contents OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/abc/all_abstract_methods_required.py`.
#[test]
fn test_gen_behavior_std_libs_abc_all_abstract_methods_required() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "behavior"
# case = "all_abstract_methods_required"
# subject = "abc.abstractmethod"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""abc.abstractmethod: with multiple abstract methods, every one must be implemented before the class is instantiable"""
import abc


class Multi(abc.ABC):
    @abc.abstractmethod
    def m1(self) -> int: ...
    @abc.abstractmethod
    def m2(self) -> int: ...


# Implementing only one abstract method is not enough.
class ImplOne(Multi):
    def m1(self) -> int:
        return 1


_raised = False
try:
    ImplOne()
except TypeError:
    _raised = True
assert _raised, "missing m2 keeps the class abstract"

# Implementing both makes it concrete.
class ImplBoth(Multi):
    def m1(self) -> int:
        return 1
    def m2(self) -> int:
        return 2


both = ImplBoth()
assert both.m1() == 1 and both.m2() == 2, "both abstract methods implemented"

print("all_abstract_methods_required OK")
"###);
    assert_output(&out, r###"all_abstract_methods_required OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/abc/full_concrete_instantiable.py`.
#[test]
fn test_gen_behavior_std_libs_abc_full_concrete_instantiable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "behavior"
# case = "full_concrete_instantiable"
# subject = "abc.ABC"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""abc.ABC: a subclass that implements the abstractmethod is concrete, instantiable, and has an empty __abstractmethods__"""
import abc


class Base(abc.ABC):
    @abc.abstractmethod
    def do(self) -> int: ...


class FullConcrete(Base):
    def do(self) -> int:
        return 42


assert FullConcrete().do() == 42, "concrete do() returns 42"
assert len(FullConcrete.__abstractmethods__) == 0, "concrete class has no abstract methods"

print("full_concrete_instantiable OK")
"###);
    assert_output(&out, r###"full_concrete_instantiable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/abc/get_cache_token_returns_int.py`.
#[test]
fn test_gen_behavior_std_libs_abc_get_cache_token_returns_int() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "behavior"
# case = "get_cache_token_returns_int"
# subject = "abc.get_cache_token"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""abc.get_cache_token: get_cache_token() returns an int per the CPython contract"""
import abc

tok = abc.get_cache_token()
assert isinstance(tok, int), f"get_cache_token returns int, got {type(tok).__name__}"

print("get_cache_token_returns_int OK")
"###);
    assert_output(&out, r###"get_cache_token_returns_int OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/abc/isinstance_issubclass_via_inheritance.py`.
#[test]
fn test_gen_behavior_std_libs_abc_isinstance_issubclass_via_inheritance() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "behavior"
# case = "isinstance_issubclass_via_inheritance"
# subject = "abc.ABC"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""abc.ABC: a real subclass of an ABC passes isinstance and issubclass against the ABC base"""
import abc


class Shape(abc.ABC):
    @abc.abstractmethod
    def area(self) -> float: ...


class Circle(Shape):
    def __init__(self, r: float):
        self.r = r
    def area(self) -> float:
        return 3.141592653589793 * self.r * self.r


c = Circle(1.0)
assert abs(c.area() - 3.141592653589793) < 1e-10, f"circle area: {c.area()!r}"
assert isinstance(c, Shape), "Circle instance isinstance Shape"
assert issubclass(Circle, Shape), "Circle issubclass Shape"

print("isinstance_issubclass_via_inheritance OK")
"###);
    assert_output(&out, r###"isinstance_issubclass_via_inheritance OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/abc/partial_concrete_still_abstract.py`.
#[test]
fn test_gen_behavior_std_libs_abc_partial_concrete_still_abstract() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "behavior"
# case = "partial_concrete_still_abstract"
# subject = "abc.ABC"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""abc.ABC: a subclass that does not override the abstractmethod is still abstract and not instantiable"""
import abc


class Base(abc.ABC):
    @abc.abstractmethod
    def do(self) -> int: ...


class PartialConcrete(Base):
    pass  # does not implement do()


# The abstract method is inherited, so the subclass is still abstract.
assert "do" in PartialConcrete.__abstractmethods__, "do still abstract in subclass"

_raised = False
try:
    PartialConcrete()
except TypeError:
    _raised = True
assert _raised, "partial concrete subclass still raises TypeError"

print("partial_concrete_still_abstract OK")
"###);
    assert_output(&out, r###"partial_concrete_still_abstract OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/abc/register_virtual_subclass.py`.
#[test]
fn test_gen_behavior_std_libs_abc_register_virtual_subclass() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "behavior"
# case = "register_virtual_subclass"
# subject = "abc.ABCMeta.register"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""abc.ABCMeta.register: register() makes an unrelated class a virtual subclass for issubclass and isinstance without inheritance"""
import abc


class Interface(abc.ABC):
    pass


class Impl:
    pass  # no inheritance relationship to Interface


# Before registration, Impl is unrelated.
assert not issubclass(Impl, Interface), "Impl is not a subclass before register"

Interface.register(Impl)
assert issubclass(Impl, Interface), "registered class is a virtual subclass"
assert isinstance(Impl(), Interface), "registered instance passes isinstance"

print("register_virtual_subclass OK")
"###);
    assert_output(&out, r###"register_virtual_subclass OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/abc/subclasshook_structural_check.py`.
#[test]
fn test_gen_behavior_std_libs_abc_subclasshook_structural_check() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "behavior"
# case = "subclasshook_structural_check"
# subject = "abc.ABCMeta"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""abc.ABCMeta: a custom __subclasshook__ drives structural issubclass checks (has __len__ -> Sized) returning NotImplemented to fall through"""
import abc


class Sized(abc.ABC):
    @classmethod
    def __subclasshook__(cls, C):
        if cls is Sized:
            return hasattr(C, "__len__")
        return NotImplemented


# Anything with __len__ is structurally a Sized.
assert issubclass(list, Sized), "list has __len__ -> Sized"
assert issubclass(str, Sized), "str has __len__ -> Sized"
assert not issubclass(int, Sized), "int has no __len__ -> not Sized"
assert isinstance([1, 2], Sized), "list instance is Sized"
assert not isinstance(7, Sized), "int instance is not Sized"

print("subclasshook_structural_check OK")
"###);
    assert_output(&out, r###"subclasshook_structural_check OK
"###);
}
