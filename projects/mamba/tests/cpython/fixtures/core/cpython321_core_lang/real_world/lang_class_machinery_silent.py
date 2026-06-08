# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_class_machinery_silent"
# subject = "cpython321.lang_class_machinery_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_class_machinery_silent.py"
# status = "filled"
# ///
"""cpython321.lang_class_machinery_silent: execute CPython 3.12 seed lang_class_machinery_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of classmethod `cls()` construction (the
# documented "@classmethod that returns cls() builds an instance of
# the calling class" — mamba returns None instead), `__new__`-set
# attribute persistence (the documented "attrs set on the instance
# inside __new__ survive into __init__/the post-construct object" —
# mamba drops the __new__-side attr and returns None), `O.inst_attr`
# raised access (the documented "accessing an instance-only attr via
# the class raises AttributeError" — mamba returns None silently),
# `obj.__dict__` exposure (the documented "instance.__dict__ exposes
# the instance attribute mapping" — mamba returns None), class
# `__bases__` tuple (the documented "cls.__bases__ is the tuple of
# direct base classes" — mamba returns None), `cls.__subclasses__()`
# (the documented "type method that lists direct subclasses" — mamba
# raises AttributeError on 'str' object), `type(name, bases, ns)`
# class-attribute install (the documented "third-arg dict installs
# class attributes" — mamba returns None for the installed attr),
# `instance.__class__` (the documented "every instance carries
# __class__ pointing to its type" — mamba returns None), property
# without setter raised assignment (the documented "assigning to a
# getter-only property raises AttributeError" — mamba silently
# accepts the assignment), and `__iadd__` return-value identity
# (the documented "a += other rebinds a to the value returned by
# __iadd__" — mamba sets a to None).
# Ten-pack pinned to atomic 325.
#
# Behavioral edges that CONFORM on mamba (basic class, instance/
# method/class attrs, isinstance/type/class name, inheritance + super
# (), diamond inheritance MRO via method dispatch, value dunders
# repr/str/eq/hash, container dunders add/mul/len/bool/contains/iter
# /getitem/setitem/lt, properties getter+setter, classmethod and
# staticmethod via Q.fn() and Q().fn() (no cls() instantiation),
# __slots__ enforcement, manual abstract method via
# NotImplementedError, instance/class attr access, getattr/setattr/
# hasattr/delattr, vars(), class variable shadowing, multiple
# inheritance dispatch order, dir() listing) are covered in the
# matching pass fixture `test_lang_class_machinery_value_ops`.


_ledger: list[int] = []

# 1) classmethod returning cls() instantiates the calling class
#    (mamba: cls() returns None — type(Q.make()).__name__ == "NoneType")
class _Q:
    @classmethod
    def make(cls):
        return cls()
assert type(_Q.make()).__name__ == "_Q"; _ledger.append(1)

# 2) __new__-set attribute survives into the post-construct object
#    (mamba: instance.created_by_new becomes None after __init__)
class _N:
    def __new__(cls):
        instance = super().__new__(cls)
        instance.created_by_new = True
        return instance
    def __init__(self):
        self.initialized = True
_n = _N()
assert _n.created_by_new == True; _ledger.append(1)

# 3) accessing instance-only attr via the class raises AttributeError
#    (mamba: returns None silently)
class _O:
    cls_attr = "shared"
    def __init__(self):
        self.inst_attr = "private"
try:
    _O.inst_attr
    raise AssertionError("expected AttributeError")
except AttributeError:
    _ledger.append(1)

# 4) instance.__dict__ exposes the instance attribute mapping
#    (mamba: returns None)
class _W:
    def __init__(self):
        self.a = 1
        self.b = 2
_w = _W()
assert _w.__dict__ == {"a": 1, "b": 2}; _ledger.append(1)

# 5) cls.__bases__ is the tuple of direct bases
#    (mamba: returns None)
class _Foo:
    pass
class _Bar(_Foo):
    pass
assert _Bar.__bases__ == (_Foo,); _ledger.append(1)

# 6) cls.__subclasses__() lists direct subclasses
#    (mamba: AttributeError 'str' object has no attribute '__subclasses__')
assert _Foo.__subclasses__() == [_Bar]; _ledger.append(1)

# 7) type(name, bases, ns) installs class attributes from the dict
#    (mamba: NewCls.x is None)
_NewCls = type("_NewCls", (), {"x": 10})
assert _NewCls.x == 10; _ledger.append(1)

# 8) instance.__class__ points to the instance's type
#    (mamba: returns None)
class _CC:
    pass
_cc = _CC()
assert _cc.__class__ is _CC; _ledger.append(1)

# 9) assigning to a getter-only property raises AttributeError
#    (mamba: silently accepts the assignment)
class _PR:
    def __init__(self):
        self._v = 5
    @property
    def v(self):
        return self._v
_pr = _PR()
try:
    _pr.v = 99
    raise AssertionError("expected AttributeError")
except AttributeError:
    _ledger.append(1)

# 10) `a += other` rebinds a to __iadd__'s return value
#     (mamba: a becomes None)
class _Acc:
    def __init__(self):
        self.items = []
    def __iadd__(self, other):
        self.items.extend(other)
        return self
_a = _Acc()
_a += [1, 2, 3]
assert type(_a).__name__ == "_Acc"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_class_machinery_silent {sum(_ledger)} asserts")
