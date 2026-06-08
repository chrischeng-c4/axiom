# Regression: two independent bugs that conspire to break a very common
# Python pattern — a container class whose `__iter__` returns a separate
# iterator class.
#
# 1. Forward-referenced class names resolved inside method bodies: when
#    `Container.__iter__` refers to `ContainerIterator` defined below,
#    the call silently returned None because `ContainerIterator` wasn't
#    yet in `user_class_syms` at method-compile time.
# 2. StopIteration raised by a user-defined `__next__` set the
#    thread-local CURRENT_EXCEPTION but the for-loop protocol only
#    consumed the iterator's STOP_ITERATION flag, so the normally-mute
#    exit from the loop leaked a StopIteration to the enclosing scope.


class Container:
    def __init__(self, items):
        self._items = items

    def __iter__(self):
        return ContainerIterator(self._items)


class ContainerIterator:
    def __init__(self, items):
        self._items = items
        self._idx = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self._idx >= len(self._items):
            raise StopIteration
        v = self._items[self._idx]
        self._idx += 1
        return v


c = Container([1, 2, 3])

# for-in, list(), tuple(), membership — all go through mb_iter +
# mb_has_next + mb_next. All must finish cleanly without leaking
# StopIteration.
for x in c:
    print("for:", x)

print("list:", list(Container([4, 5, 6])))
print("tuple:", tuple(Container([7, 8, 9])))

# Sequence unpacking uses mb_list_from_iterable → same iterator path.
a, b, d = Container([10, 20, 30])
print("unpack:", a, b, d)

# Membership test builds a fresh iterator each call.
print("2 in c:", 2 in c)
print("5 in c:", 5 in c)

# Comprehensions, sum, any, all all exercise the protocol.
print([x * 10 for x in Container([1, 2, 3])])
print("sum:", sum(Container([1, 2, 3])))
print("any:", any(Container([0, 0, 1])))
print("all:", all(Container([1, 2, 3])))

# Bonus: both classes remain usable after the for-loop exit.
# If StopIteration leaked, this line would never execute.
print("after loop: ok")
