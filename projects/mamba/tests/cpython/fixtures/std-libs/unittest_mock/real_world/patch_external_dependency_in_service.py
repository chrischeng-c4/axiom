# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "real_world"
# case = "patch_external_dependency_in_service"
# subject = "unittest.mock.patch"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testpatch.py"
# status = "filled"
# ///
"""unittest.mock.patch: a service that calls an external dependency is unit-tested by patching that dependency with a configured mock, asserting the service result and that the dependency was called with the expected arguments"""
from unittest.mock import patch


class Repo:
    def fetch(self, key):  # a real impl would hit an external store
        raise RuntimeError("real backend access is not allowed in a unit test")


class Service:
    def __init__(self, repo):
        self.repo = repo

    def describe(self, key):
        row = self.repo.fetch(key)
        return f"{key}={row}"


repo = Repo()
svc = Service(repo)
with patch.object(repo, "fetch", return_value="VALUE") as fetch_mock:
    result = svc.describe("k1")
assert result == "k1=VALUE"
fetch_mock.assert_called_once_with("k1")
print("patch_external_dependency_in_service OK")
