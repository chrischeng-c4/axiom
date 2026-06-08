# Descriptor protocol — __get__, __set__, __delete__ — #2785.
#
# Covers Python's descriptor protocol. A descriptor is any object
# whose class defines __get__ (and optionally __set__ / __delete__).
# Attribute access on an instance routes through the descriptor
# protocol when the attribute is found on the type (not the
# instance) and the type-side object is a descriptor.
#
# Precedence rules (CPython docs / data model):
#
#   1. Data descriptors      (have __set__ or __delete__)
#   2. Instance __dict__
#   3. Non-data descriptors  (only __get__)
#
# So a DATA descriptor wins over an instance attribute of the same
# name, but an instance attribute wins over a NON-DATA descriptor.
# This precedence is what makes @property a data descriptor and
# regular functions / classmethod-decorated callables non-data
# descriptors.
#
# Clauses:
#   1. __get__ called on instance access — receives `(instance,
#      owner)`. On class access, `instance` is None.
#   2. __set__ called on assignment.
#   3. __delete__ called on `del`.
#   4. Data descriptor wins over instance __dict__ — even if the
#      instance dict has the same key, the descriptor's __get__
#      runs.
#   5. Non-data descriptor LOSES to instance __dict__ — once the
#      instance dict has the key, attribute access returns the
#      dict value, not the descriptor.
#   6. Class access bypasses the descriptor for non-data descriptors
#      that return self (`Class.attr` returns the descriptor itself,
#      not the value).
#
# Every print line tagged `[descriptor]` so failure output names
# descriptor semantics.


class DataDesc:
    """Data descriptor — defines __set__/__delete__ so it WINS over
    any instance __dict__ entry of the same name."""

    def __init__(self, default):
        self.default = default
        self.get_log = []
        self.set_log = []
        self.delete_log = []

    def __set_name__(self, _owner, name):
        self._slot = "_" + name

    def __get__(self, instance, owner):
        # owner is the class; instance is None on class access.
        if instance is None:
            self.get_log.append(("class", owner.__name__))
            return self
        self.get_log.append(("instance", id(instance)))
        return instance.__dict__.get(self._slot, self.default)

    def __set__(self, instance, value):
        self.set_log.append(("instance", value))
        instance.__dict__[self._slot] = value

    def __delete__(self, instance):
        self.delete_log.append("instance")
        instance.__dict__.pop(self._slot, None)


class NonDataDesc:
    """Non-data descriptor — only __get__. Loses to instance
    __dict__ entry of the same name."""

    def __init__(self, default):
        self.default = default
        self.get_log = []

    def __get__(self, instance, owner):
        if instance is None:
            self.get_log.append(("class", owner.__name__))
            return self
        self.get_log.append(("instance", id(instance)))
        return self.default


class Holder:
    data = DataDesc("data-default")
    nondata = NonDataDesc("nondata-default")


h = Holder()


# Clause 1: __get__ dispatch on instance access.
print("[descriptor] clause-1 instance-get:", h.data)
# Class access — instance is None, returns the descriptor itself.
fetched = Holder.data
print("[descriptor] clause-1 class-get-self:", fetched is Holder.__dict__["data"])
# get_log captured the (class, ...) entry.
last = Holder.__dict__["data"].get_log[-1]
print("[descriptor] clause-1 class-get-log:", last[0])


# Clause 2: __set__ dispatch.
h.data = "value-A"
print("[descriptor] clause-2 set-log:", Holder.__dict__["data"].set_log[-1])
print("[descriptor] clause-2 get-after-set:", h.data)


# Clause 3: __delete__ dispatch.
del h.data
print("[descriptor] clause-3 delete-log:", Holder.__dict__["data"].delete_log[-1])
# After deletion the descriptor's default is back.
print("[descriptor] clause-3 get-after-delete:", h.data)


# Clause 4: data descriptor wins over instance __dict__. We poke a
# value DIRECTLY into the instance dict (bypassing __set__); the
# descriptor still wins on read because it's a data descriptor.
h.__dict__["data"] = "smuggled"
print("[descriptor] clause-4 dict-has-smuggled:", h.__dict__["data"])
print("[descriptor] clause-4 attr-returns-descriptor-default:", h.data)


# Clause 5: non-data descriptor LOSES to instance __dict__. Smuggle
# a value into the instance dict; attribute access returns the dict
# entry, not the descriptor's default.
print("[descriptor] clause-5 descriptor-default:", h.nondata)
h.__dict__["nondata"] = "smuggled-nondata"
print("[descriptor] clause-5 instance-wins:", h.nondata)


# Clause 6: instance vs class access of a non-data descriptor that
# returns `self` on class access. The descriptor's __get__ runs in
# both cases but with different `instance` value.
class FuncLike:
    def __get__(self, instance, owner):
        if instance is None:
            return ("class-access", owner.__name__)
        return ("instance-access", id(instance))


class HasFn:
    fn = FuncLike()


hf = HasFn()
print("[descriptor] clause-6 instance-access:", hf.fn[0])
print("[descriptor] clause-6 class-access:", HasFn.fn[0])
print("[descriptor] clause-6 class-access-owner:", HasFn.fn[1])
