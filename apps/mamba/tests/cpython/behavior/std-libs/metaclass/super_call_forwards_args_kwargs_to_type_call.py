# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "metaclass"
# dimension = "behavior"
# case = "super_call_forwards_args_kwargs_to_type_call"
# subject = "type.__call__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_metaclass.py"
# status = "filled"
# ///
"""type.__call__: a metaclass __call__ override that forwards via super().__call__(*args, **kwargs) binds kwargs by name into __init__ (type.__call__'s default instance-creation bypass), for both an all-positional-plus-kwarg call and a call relying on defaults (#1951)"""

_calls = []


class Meta(type):
    def __call__(cls, *args, **kwargs):
        _calls.append((args, kwargs))
        return super().__call__(*args, **kwargs)


class C(metaclass=Meta):
    def __init__(self, x, y=0, *, z=5):
        self.x = x
        self.y = y
        self.z = z


c1 = C(1, 2, z=9)
assert _calls[-1] == ((1, 2), {"z": 9}), _calls[-1]
assert (c1.x, c1.y, c1.z) == (1, 2, 9), (c1.x, c1.y, c1.z)

c2 = C(7)
assert _calls[-1] == ((7,), {}), _calls[-1]
assert (c2.x, c2.y, c2.z) == (7, 0, 5), (c2.x, c2.y, c2.z)

print("super_call_forwards_args_kwargs_to_type_call OK")
