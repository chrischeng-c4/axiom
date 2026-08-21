# @property with deleter

class Resource:
    def __init__(self, name):
        self._name = name
        self.deleted = False

    @property
    def name(self):
        return self._name

    @name.setter
    def name(self, value):
        self._name = value

    @name.deleter
    def name(self):
        self._name = None
        self.deleted = True

r = Resource("foo")
print(r.name)
r.name = "bar"
print(r.name)

del r.name
print(r.name)
print(r.deleted)
