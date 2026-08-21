# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_user_data"
# subject = "cpython.test_minidom.MinidomTest.testUserData"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testUserData
"""Auto-ported test: MinidomTest::testUserData (CPython 3.12 oracle)."""


from xml.dom.minidom import Document


class UserDataHandler:
    called = 0

    def handle(self, operation, key, data, src, dst):
        dst.setUserData(key, data + 1, self)
        src.setUserData(key, None, None)
        self.called = 1


dom = Document()
n = dom.createElement("e")
assert n.getUserData("foo") is None
n.setUserData("foo", None, None)
assert n.getUserData("foo") is None
n.setUserData("foo", 12, 12)
n.setUserData("bar", 13, 13)
assert n.getUserData("foo") == 12
assert n.getUserData("bar") == 13
n.setUserData("foo", None, None)
assert n.getUserData("foo") is None
assert n.getUserData("bar") == 13

handler = UserDataHandler()
n.setUserData("bar", 12, handler)
c = n.cloneNode(1)
assert handler.called
assert n.getUserData("bar") is None
assert c.getUserData("bar") == 13
n.unlink()
c.unlink()
dom.unlink()

print("ok")
