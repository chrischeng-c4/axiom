# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Decorator evaluation order (CPython data model). Stacking has two phases:
#   1. Decorator EXPRESSIONS (and their args) are evaluated top-to-bottom,
#      before the decorated function exists -- for `@a.make(a.arg)`: look up
#      `a.make`, evaluate `a.arg`, then call the factory.
#   2. The resulting decorators are APPLIED bottom-to-top (innermost first).
# A tracer records every name lookup and decorator call as a flat list.

actions = []


def make_decorator(tag):
    actions.append("makedec" + tag)

    def decorate(func):
        actions.append("calldec" + tag)
        return func
    return decorate


class Tracer:
    def __init__(self, index):
        self.index = index

    def __getattr__(self, name):
        if name == "make_decorator":
            op, res = "evalname", make_decorator
        elif name == "arg":
            op, res = "evalargs", str(self.index)
        else:
            raise AssertionError("unknown attr %s" % name)
        actions.append("%s%d" % (op, self.index))
        return res


c1, c2, c3 = Tracer(1), Tracer(2), Tracer(3)

expected = [
    # phase 1: expressions + args evaluated top-down, factories build decorators
    "evalname1", "evalargs1", "makedec1",
    "evalname2", "evalargs2", "makedec2",
    "evalname3", "evalargs3", "makedec3",
    # phase 2: decorators applied bottom-up
    "calldec3", "calldec2", "calldec1",
]


@c1.make_decorator(c1.arg)
@c2.make_decorator(c2.arg)
@c3.make_decorator(c3.arg)
def foo():
    return 42


assert foo() == 42
assert actions == expected, actions


# The decorator-syntax order is exactly equivalent to manual nesting: applying
# the factories outside-in by hand reproduces the same trace.
actions.clear()


def bar():
    return 42


bar = c1.make_decorator(c1.arg)(
    c2.make_decorator(c2.arg)(
        c3.make_decorator(c3.arg)(bar)))

assert bar() == 42
assert actions == expected, actions

print("eval_order OK")
