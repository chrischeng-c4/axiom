use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/platform/comparable_version_ordering.py`.
#[test]
fn test_gen_behavior_std_libs_platform_comparable_version_ordering() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "comparable_version_ordering"
# subject = "platform._comparable_version"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform._comparable_version: _comparable_version sorts version strings: numeric segments compare numerically, separators (. _ - +) normalize, and the pre-release ladder is dev<alpha<beta<candidate<final<post"""
import platform

V = platform._comparable_version

# Equal strings compare equal; numeric segments sort numerically.
assert V("1.2.3") == V("1.2.3"), "identical strings equal"
assert V("8.02") == V("8.02"), "leading zero kept stable"
assert V("1.2.3") < V("1.2.10"), "10 sorts after 3 numerically"
assert V("0.9") < V("2.2"), "major number dominates"

# Mixed separators (. _ - +) normalize to the same key.
assert V("1.2.3.4") == V("1_2-3+4"), "separators normalize"

# Pre-release ladder: dev < alpha/a < beta/b < candidate/rc < final < post.
assert V("1.2dev") < V("1.2alpha"), "dev before alpha"
assert V("1.2alpha") < V("1.2beta"), "alpha before beta"
assert V("1.2a") < V("1.2b"), "a before b"
assert V("1.2b") < V("1.2c"), "b before c"
assert V("1.2c") < V("1.2rc"), "c before rc"
assert V("1.2rc") < V("1.2.0"), "rc before final release"
assert V("1.2.0") < V("1.2pl"), "final before post-level"

# More-specific version sorts after its prefix.
assert V("1.2") < V("1.2.1"), "1.2 before 1.2.1"
assert V("0.4") < V("0.4.0"), "bare before zero-padded"

print("comparable_version_ordering OK")
"###);
    assert_output(&out, r###"comparable_version_ordering OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/platform/machine_returns_nonempty_str.py`.
#[test]
fn test_gen_behavior_std_libs_platform_machine_returns_nonempty_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "machine_returns_nonempty_str"
# subject = "platform.machine"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform.machine: machine() returns a non-empty str naming the host architecture (value host-dependent, only shape asserted)"""
import platform

out = platform.machine()
assert type(out).__name__ == "str", "machine() returns str"
assert len(out) > 0, "machine() is non-empty"

print("machine_returns_nonempty_str OK")
"###);
    assert_output(&out, r###"machine_returns_nonempty_str OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/platform/node_returns_str.py`.
#[test]
fn test_gen_behavior_std_libs_platform_node_returns_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "node_returns_str"
# subject = "platform.node"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform.node: node() returns a str hostname (may be empty, so only type is asserted)"""
import platform

out = platform.node()
assert type(out).__name__ == "str", "node() returns str"

print("node_returns_str OK")
"###);
    assert_output(&out, r###"node_returns_str OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/platform/parse_os_release_parsing.py`.
#[test]
fn test_gen_behavior_std_libs_platform_parse_os_release_parsing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "parse_os_release_parsing"
# subject = "platform._parse_os_release"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform._parse_os_release: _parse_os_release strips quotes, resolves shell escapes, skips comment/blank/malformed lines, and supplies default ID/NAME/PRETTY_NAME"""
import platform

FEDORA = (
    'NAME=Fedora\n'
    'VERSION="32 (Thirty Two)"\n'
    'ID=fedora\n'
    'VERSION_CODENAME=""\n'
)
info = platform._parse_os_release(FEDORA.splitlines())
assert info["NAME"] == "Fedora", "unquoted value"
assert info["ID"] == "fedora", "lowercase id"
assert info["VERSION_CODENAME"] == "", "empty quoted value -> empty string"
assert "ID_LIKE" not in info, "absent key is missing, not blank"

UBUNTU = (
    'NAME="Ubuntu"\n'
    'ID=ubuntu\n'
    'ID_LIKE=debian\n'
    'VERSION_CODENAME=focal\n'
)
info = platform._parse_os_release(UBUNTU.splitlines())
assert info["NAME"] == "Ubuntu", "quoted value stripped"
assert info["ID_LIKE"] == "debian", "id_like preserved"
assert info["VERSION_CODENAME"] == "focal", "bare value preserved"

# Comments, blanks, and malformed lines are ignored; quoting + escapes resolve.
TRICKY = (
    '\n'
    '# comment line\n'
    'ID_LIKE="egg spam viking"\n'
    'EMPTY=\n'
    "SINGLE_QUOTE='single'\n"
    'DOUBLE_QUOTE="double"\n'
    'QUOTES="double\\\'s"\n'
    'SPECIALS="\\$\\`\\\\\\\'\\""\n'
    '=invalid\n'
    'INVALID\n'
    'IN-VALID=value\n'
)
info = platform._parse_os_release(TRICKY.splitlines())
assert info["ID"] == "linux", "default ID when unspecified"
assert info["NAME"] == "Linux", "default NAME"
assert info["PRETTY_NAME"] == "Linux", "default PRETTY_NAME"
assert info["ID_LIKE"] == "egg spam viking", "spaces inside quotes kept"
assert info["EMPTY"] == "", "bare KEY= is empty string"
assert info["SINGLE_QUOTE"] == "single", "single quotes stripped"
assert info["DOUBLE_QUOTE"] == "double", "double quotes stripped"
assert info["QUOTES"] == "double's", "escaped apostrophe resolved"
assert info["SPECIALS"] == '$`\\\'"', "shell escapes resolved"
assert len(info["SPECIALS"]) == 5, "five resolved special chars"
assert "IN-VALID" not in info, "key with dash rejected"

print("parse_os_release_parsing OK")
"###);
    assert_output(&out, r###"parse_os_release_parsing OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/platform/platform_returns_str.py`.
#[test]
fn test_gen_behavior_std_libs_platform_platform_returns_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "platform_returns_str"
# subject = "platform.platform"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform.platform: platform() returns a str summary of the platform (value host-dependent, only type asserted)"""
import platform

out = platform.platform()
assert type(out).__name__ == "str", "platform() returns str"

print("platform_returns_str OK")
"###);
    assert_output(&out, r###"platform_returns_str OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/platform/platform_test__test_architecture.py`.
#[test]
fn test_gen_behavior_std_libs_platform_platform_test__test_architecture() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "platform_test__test_architecture"
# subject = "cpython.test_platform.PlatformTest.test_architecture"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_platform.py::PlatformTest::test_architecture
"""Auto-ported test: PlatformTest::test_architecture (CPython 3.12 oracle)."""


import os
import copy
import pickle
import platform
import subprocess
import sys
import unittest
from unittest import mock
from test import support
from test.support import os_helper


FEDORA_OS_RELEASE = 'NAME=Fedora\nVERSION="32 (Thirty Two)"\nID=fedora\nVERSION_ID=32\nVERSION_CODENAME=""\nPLATFORM_ID="platform:f32"\nPRETTY_NAME="Fedora 32 (Thirty Two)"\nANSI_COLOR="0;34"\nLOGO=fedora-logo-icon\nCPE_NAME="cpe:/o:fedoraproject:fedora:32"\nHOME_URL="https://fedoraproject.org/"\nDOCUMENTATION_URL="https://docs.fedoraproject.org/en-US/fedora/f32/system-administrators-guide/"\nSUPPORT_URL="https://fedoraproject.org/wiki/Communicating_and_getting_help"\nBUG_REPORT_URL="https://bugzilla.redhat.com/"\nREDHAT_BUGZILLA_PRODUCT="Fedora"\nREDHAT_BUGZILLA_PRODUCT_VERSION=32\nREDHAT_SUPPORT_PRODUCT="Fedora"\nREDHAT_SUPPORT_PRODUCT_VERSION=32\nPRIVACY_POLICY_URL="https://fedoraproject.org/wiki/Legal:PrivacyPolicy"\n'

UBUNTU_OS_RELEASE = 'NAME="Ubuntu"\nVERSION="20.04.1 LTS (Focal Fossa)"\nID=ubuntu\nID_LIKE=debian\nPRETTY_NAME="Ubuntu 20.04.1 LTS"\nVERSION_ID="20.04"\nHOME_URL="https://www.ubuntu.com/"\nSUPPORT_URL="https://help.ubuntu.com/"\nBUG_REPORT_URL="https://bugs.launchpad.net/ubuntu/"\nPRIVACY_POLICY_URL="https://www.ubuntu.com/legal/terms-and-policies/privacy-policy"\nVERSION_CODENAME=focal\nUBUNTU_CODENAME=focal\n'

TEST_OS_RELEASE = '\n# test data\nID_LIKE="egg spam viking"\nEMPTY=\n# comments and empty lines are ignored\n\nSINGLE_QUOTE=\'single\'\nEMPTY_SINGLE=\'\'\nDOUBLE_QUOTE="double"\nEMPTY_DOUBLE=""\nQUOTES="double\\\'s"\nSPECIALS="\\$\\`\\\\\\\'\\""\n# invalid lines\n=invalid\n=\nINVALID\nIN-VALID=value\nIN VALID=value\n'


# --- test body ---
self_save_version = sys.version
self_save_git = sys._git
self_save_platform = sys.platform
res = platform.architecture()
print("PlatformTest::test_architecture: ok")
"###);
    assert_output(&out, r###"PlatformTest::test_architecture: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/platform/platform_test__test_java_ver.py`.
#[test]
fn test_gen_behavior_std_libs_platform_platform_test__test_java_ver() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "platform_test__test_java_ver"
# subject = "cpython.test_platform.PlatformTest.test_java_ver"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_platform.py::PlatformTest::test_java_ver
"""Auto-ported test: PlatformTest::test_java_ver (CPython 3.12 oracle)."""


import os
import copy
import pickle
import platform
import subprocess
import sys
import unittest
from unittest import mock
from test import support
from test.support import os_helper


FEDORA_OS_RELEASE = 'NAME=Fedora\nVERSION="32 (Thirty Two)"\nID=fedora\nVERSION_ID=32\nVERSION_CODENAME=""\nPLATFORM_ID="platform:f32"\nPRETTY_NAME="Fedora 32 (Thirty Two)"\nANSI_COLOR="0;34"\nLOGO=fedora-logo-icon\nCPE_NAME="cpe:/o:fedoraproject:fedora:32"\nHOME_URL="https://fedoraproject.org/"\nDOCUMENTATION_URL="https://docs.fedoraproject.org/en-US/fedora/f32/system-administrators-guide/"\nSUPPORT_URL="https://fedoraproject.org/wiki/Communicating_and_getting_help"\nBUG_REPORT_URL="https://bugzilla.redhat.com/"\nREDHAT_BUGZILLA_PRODUCT="Fedora"\nREDHAT_BUGZILLA_PRODUCT_VERSION=32\nREDHAT_SUPPORT_PRODUCT="Fedora"\nREDHAT_SUPPORT_PRODUCT_VERSION=32\nPRIVACY_POLICY_URL="https://fedoraproject.org/wiki/Legal:PrivacyPolicy"\n'

UBUNTU_OS_RELEASE = 'NAME="Ubuntu"\nVERSION="20.04.1 LTS (Focal Fossa)"\nID=ubuntu\nID_LIKE=debian\nPRETTY_NAME="Ubuntu 20.04.1 LTS"\nVERSION_ID="20.04"\nHOME_URL="https://www.ubuntu.com/"\nSUPPORT_URL="https://help.ubuntu.com/"\nBUG_REPORT_URL="https://bugs.launchpad.net/ubuntu/"\nPRIVACY_POLICY_URL="https://www.ubuntu.com/legal/terms-and-policies/privacy-policy"\nVERSION_CODENAME=focal\nUBUNTU_CODENAME=focal\n'

TEST_OS_RELEASE = '\n# test data\nID_LIKE="egg spam viking"\nEMPTY=\n# comments and empty lines are ignored\n\nSINGLE_QUOTE=\'single\'\nEMPTY_SINGLE=\'\'\nDOUBLE_QUOTE="double"\nEMPTY_DOUBLE=""\nQUOTES="double\\\'s"\nSPECIALS="\\$\\`\\\\\\\'\\""\n# invalid lines\n=invalid\n=\nINVALID\nIN-VALID=value\nIN VALID=value\n'


# --- test body ---
self_save_version = sys.version
self_save_git = sys._git
self_save_platform = sys.platform
res = platform.java_ver()
if sys.platform == 'java':

    assert all(res)
print("PlatformTest::test_java_ver: ok")
"###);
    assert_output(&out, r###"PlatformTest::test_java_ver: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/platform/platform_test__test_machine.py`.
#[test]
fn test_gen_behavior_std_libs_platform_platform_test__test_machine() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "platform_test__test_machine"
# subject = "cpython.test_platform.PlatformTest.test_machine"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_platform.py::PlatformTest::test_machine
"""Auto-ported test: PlatformTest::test_machine (CPython 3.12 oracle)."""


import os
import copy
import pickle
import platform
import subprocess
import sys
import unittest
from unittest import mock
from test import support
from test.support import os_helper


FEDORA_OS_RELEASE = 'NAME=Fedora\nVERSION="32 (Thirty Two)"\nID=fedora\nVERSION_ID=32\nVERSION_CODENAME=""\nPLATFORM_ID="platform:f32"\nPRETTY_NAME="Fedora 32 (Thirty Two)"\nANSI_COLOR="0;34"\nLOGO=fedora-logo-icon\nCPE_NAME="cpe:/o:fedoraproject:fedora:32"\nHOME_URL="https://fedoraproject.org/"\nDOCUMENTATION_URL="https://docs.fedoraproject.org/en-US/fedora/f32/system-administrators-guide/"\nSUPPORT_URL="https://fedoraproject.org/wiki/Communicating_and_getting_help"\nBUG_REPORT_URL="https://bugzilla.redhat.com/"\nREDHAT_BUGZILLA_PRODUCT="Fedora"\nREDHAT_BUGZILLA_PRODUCT_VERSION=32\nREDHAT_SUPPORT_PRODUCT="Fedora"\nREDHAT_SUPPORT_PRODUCT_VERSION=32\nPRIVACY_POLICY_URL="https://fedoraproject.org/wiki/Legal:PrivacyPolicy"\n'

UBUNTU_OS_RELEASE = 'NAME="Ubuntu"\nVERSION="20.04.1 LTS (Focal Fossa)"\nID=ubuntu\nID_LIKE=debian\nPRETTY_NAME="Ubuntu 20.04.1 LTS"\nVERSION_ID="20.04"\nHOME_URL="https://www.ubuntu.com/"\nSUPPORT_URL="https://help.ubuntu.com/"\nBUG_REPORT_URL="https://bugs.launchpad.net/ubuntu/"\nPRIVACY_POLICY_URL="https://www.ubuntu.com/legal/terms-and-policies/privacy-policy"\nVERSION_CODENAME=focal\nUBUNTU_CODENAME=focal\n'

TEST_OS_RELEASE = '\n# test data\nID_LIKE="egg spam viking"\nEMPTY=\n# comments and empty lines are ignored\n\nSINGLE_QUOTE=\'single\'\nEMPTY_SINGLE=\'\'\nDOUBLE_QUOTE="double"\nEMPTY_DOUBLE=""\nQUOTES="double\\\'s"\nSPECIALS="\\$\\`\\\\\\\'\\""\n# invalid lines\n=invalid\n=\nINVALID\nIN-VALID=value\nIN VALID=value\n'


# --- test body ---
self_save_version = sys.version
self_save_git = sys._git
self_save_platform = sys.platform
res = platform.machine()
print("PlatformTest::test_machine: ok")
"###);
    assert_output(&out, r###"PlatformTest::test_machine: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/platform/platform_test__test_node.py`.
#[test]
fn test_gen_behavior_std_libs_platform_platform_test__test_node() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "platform_test__test_node"
# subject = "cpython.test_platform.PlatformTest.test_node"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_platform.py::PlatformTest::test_node
"""Auto-ported test: PlatformTest::test_node (CPython 3.12 oracle)."""


import os
import copy
import pickle
import platform
import subprocess
import sys
import unittest
from unittest import mock
from test import support
from test.support import os_helper


FEDORA_OS_RELEASE = 'NAME=Fedora\nVERSION="32 (Thirty Two)"\nID=fedora\nVERSION_ID=32\nVERSION_CODENAME=""\nPLATFORM_ID="platform:f32"\nPRETTY_NAME="Fedora 32 (Thirty Two)"\nANSI_COLOR="0;34"\nLOGO=fedora-logo-icon\nCPE_NAME="cpe:/o:fedoraproject:fedora:32"\nHOME_URL="https://fedoraproject.org/"\nDOCUMENTATION_URL="https://docs.fedoraproject.org/en-US/fedora/f32/system-administrators-guide/"\nSUPPORT_URL="https://fedoraproject.org/wiki/Communicating_and_getting_help"\nBUG_REPORT_URL="https://bugzilla.redhat.com/"\nREDHAT_BUGZILLA_PRODUCT="Fedora"\nREDHAT_BUGZILLA_PRODUCT_VERSION=32\nREDHAT_SUPPORT_PRODUCT="Fedora"\nREDHAT_SUPPORT_PRODUCT_VERSION=32\nPRIVACY_POLICY_URL="https://fedoraproject.org/wiki/Legal:PrivacyPolicy"\n'

UBUNTU_OS_RELEASE = 'NAME="Ubuntu"\nVERSION="20.04.1 LTS (Focal Fossa)"\nID=ubuntu\nID_LIKE=debian\nPRETTY_NAME="Ubuntu 20.04.1 LTS"\nVERSION_ID="20.04"\nHOME_URL="https://www.ubuntu.com/"\nSUPPORT_URL="https://help.ubuntu.com/"\nBUG_REPORT_URL="https://bugs.launchpad.net/ubuntu/"\nPRIVACY_POLICY_URL="https://www.ubuntu.com/legal/terms-and-policies/privacy-policy"\nVERSION_CODENAME=focal\nUBUNTU_CODENAME=focal\n'

TEST_OS_RELEASE = '\n# test data\nID_LIKE="egg spam viking"\nEMPTY=\n# comments and empty lines are ignored\n\nSINGLE_QUOTE=\'single\'\nEMPTY_SINGLE=\'\'\nDOUBLE_QUOTE="double"\nEMPTY_DOUBLE=""\nQUOTES="double\\\'s"\nSPECIALS="\\$\\`\\\\\\\'\\""\n# invalid lines\n=invalid\n=\nINVALID\nIN-VALID=value\nIN VALID=value\n'


# --- test body ---
self_save_version = sys.version
self_save_git = sys._git
self_save_platform = sys.platform
res = platform.node()
print("PlatformTest::test_node: ok")
"###);
    assert_output(&out, r###"PlatformTest::test_node: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/platform/platform_test__test_platform.py`.
#[test]
fn test_gen_behavior_std_libs_platform_platform_test__test_platform() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "platform_test__test_platform"
# subject = "cpython.test_platform.PlatformTest.test_platform"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_platform.py::PlatformTest::test_platform
"""Auto-ported test: PlatformTest::test_platform (CPython 3.12 oracle)."""


import os
import copy
import pickle
import platform
import subprocess
import sys
import unittest
from unittest import mock
from test import support
from test.support import os_helper


FEDORA_OS_RELEASE = 'NAME=Fedora\nVERSION="32 (Thirty Two)"\nID=fedora\nVERSION_ID=32\nVERSION_CODENAME=""\nPLATFORM_ID="platform:f32"\nPRETTY_NAME="Fedora 32 (Thirty Two)"\nANSI_COLOR="0;34"\nLOGO=fedora-logo-icon\nCPE_NAME="cpe:/o:fedoraproject:fedora:32"\nHOME_URL="https://fedoraproject.org/"\nDOCUMENTATION_URL="https://docs.fedoraproject.org/en-US/fedora/f32/system-administrators-guide/"\nSUPPORT_URL="https://fedoraproject.org/wiki/Communicating_and_getting_help"\nBUG_REPORT_URL="https://bugzilla.redhat.com/"\nREDHAT_BUGZILLA_PRODUCT="Fedora"\nREDHAT_BUGZILLA_PRODUCT_VERSION=32\nREDHAT_SUPPORT_PRODUCT="Fedora"\nREDHAT_SUPPORT_PRODUCT_VERSION=32\nPRIVACY_POLICY_URL="https://fedoraproject.org/wiki/Legal:PrivacyPolicy"\n'

UBUNTU_OS_RELEASE = 'NAME="Ubuntu"\nVERSION="20.04.1 LTS (Focal Fossa)"\nID=ubuntu\nID_LIKE=debian\nPRETTY_NAME="Ubuntu 20.04.1 LTS"\nVERSION_ID="20.04"\nHOME_URL="https://www.ubuntu.com/"\nSUPPORT_URL="https://help.ubuntu.com/"\nBUG_REPORT_URL="https://bugs.launchpad.net/ubuntu/"\nPRIVACY_POLICY_URL="https://www.ubuntu.com/legal/terms-and-policies/privacy-policy"\nVERSION_CODENAME=focal\nUBUNTU_CODENAME=focal\n'

TEST_OS_RELEASE = '\n# test data\nID_LIKE="egg spam viking"\nEMPTY=\n# comments and empty lines are ignored\n\nSINGLE_QUOTE=\'single\'\nEMPTY_SINGLE=\'\'\nDOUBLE_QUOTE="double"\nEMPTY_DOUBLE=""\nQUOTES="double\\\'s"\nSPECIALS="\\$\\`\\\\\\\'\\""\n# invalid lines\n=invalid\n=\nINVALID\nIN-VALID=value\nIN VALID=value\n'


# --- test body ---
self_save_version = sys.version
self_save_git = sys._git
self_save_platform = sys.platform
for aliased in (False, True):
    for terse in (False, True):
        res = platform.platform(aliased, terse)
print("PlatformTest::test_platform: ok")
"###);
    assert_output(&out, r###"PlatformTest::test_platform: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/platform/platform_test__test_processor.py`.
#[test]
fn test_gen_behavior_std_libs_platform_platform_test__test_processor() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "platform_test__test_processor"
# subject = "cpython.test_platform.PlatformTest.test_processor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_platform.py::PlatformTest::test_processor
"""Auto-ported test: PlatformTest::test_processor (CPython 3.12 oracle)."""


import os
import copy
import pickle
import platform
import subprocess
import sys
import unittest
from unittest import mock
from test import support
from test.support import os_helper


FEDORA_OS_RELEASE = 'NAME=Fedora\nVERSION="32 (Thirty Two)"\nID=fedora\nVERSION_ID=32\nVERSION_CODENAME=""\nPLATFORM_ID="platform:f32"\nPRETTY_NAME="Fedora 32 (Thirty Two)"\nANSI_COLOR="0;34"\nLOGO=fedora-logo-icon\nCPE_NAME="cpe:/o:fedoraproject:fedora:32"\nHOME_URL="https://fedoraproject.org/"\nDOCUMENTATION_URL="https://docs.fedoraproject.org/en-US/fedora/f32/system-administrators-guide/"\nSUPPORT_URL="https://fedoraproject.org/wiki/Communicating_and_getting_help"\nBUG_REPORT_URL="https://bugzilla.redhat.com/"\nREDHAT_BUGZILLA_PRODUCT="Fedora"\nREDHAT_BUGZILLA_PRODUCT_VERSION=32\nREDHAT_SUPPORT_PRODUCT="Fedora"\nREDHAT_SUPPORT_PRODUCT_VERSION=32\nPRIVACY_POLICY_URL="https://fedoraproject.org/wiki/Legal:PrivacyPolicy"\n'

UBUNTU_OS_RELEASE = 'NAME="Ubuntu"\nVERSION="20.04.1 LTS (Focal Fossa)"\nID=ubuntu\nID_LIKE=debian\nPRETTY_NAME="Ubuntu 20.04.1 LTS"\nVERSION_ID="20.04"\nHOME_URL="https://www.ubuntu.com/"\nSUPPORT_URL="https://help.ubuntu.com/"\nBUG_REPORT_URL="https://bugs.launchpad.net/ubuntu/"\nPRIVACY_POLICY_URL="https://www.ubuntu.com/legal/terms-and-policies/privacy-policy"\nVERSION_CODENAME=focal\nUBUNTU_CODENAME=focal\n'

TEST_OS_RELEASE = '\n# test data\nID_LIKE="egg spam viking"\nEMPTY=\n# comments and empty lines are ignored\n\nSINGLE_QUOTE=\'single\'\nEMPTY_SINGLE=\'\'\nDOUBLE_QUOTE="double"\nEMPTY_DOUBLE=""\nQUOTES="double\\\'s"\nSPECIALS="\\$\\`\\\\\\\'\\""\n# invalid lines\n=invalid\n=\nINVALID\nIN-VALID=value\nIN VALID=value\n'


# --- test body ---
self_save_version = sys.version
self_save_git = sys._git
self_save_platform = sys.platform
res = platform.processor()
print("PlatformTest::test_processor: ok")
"###);
    assert_output(&out, r###"PlatformTest::test_processor: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/platform/platform_test__test_release.py`.
#[test]
fn test_gen_behavior_std_libs_platform_platform_test__test_release() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "platform_test__test_release"
# subject = "cpython.test_platform.PlatformTest.test_release"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_platform.py::PlatformTest::test_release
"""Auto-ported test: PlatformTest::test_release (CPython 3.12 oracle)."""


import os
import copy
import pickle
import platform
import subprocess
import sys
import unittest
from unittest import mock
from test import support
from test.support import os_helper


FEDORA_OS_RELEASE = 'NAME=Fedora\nVERSION="32 (Thirty Two)"\nID=fedora\nVERSION_ID=32\nVERSION_CODENAME=""\nPLATFORM_ID="platform:f32"\nPRETTY_NAME="Fedora 32 (Thirty Two)"\nANSI_COLOR="0;34"\nLOGO=fedora-logo-icon\nCPE_NAME="cpe:/o:fedoraproject:fedora:32"\nHOME_URL="https://fedoraproject.org/"\nDOCUMENTATION_URL="https://docs.fedoraproject.org/en-US/fedora/f32/system-administrators-guide/"\nSUPPORT_URL="https://fedoraproject.org/wiki/Communicating_and_getting_help"\nBUG_REPORT_URL="https://bugzilla.redhat.com/"\nREDHAT_BUGZILLA_PRODUCT="Fedora"\nREDHAT_BUGZILLA_PRODUCT_VERSION=32\nREDHAT_SUPPORT_PRODUCT="Fedora"\nREDHAT_SUPPORT_PRODUCT_VERSION=32\nPRIVACY_POLICY_URL="https://fedoraproject.org/wiki/Legal:PrivacyPolicy"\n'

UBUNTU_OS_RELEASE = 'NAME="Ubuntu"\nVERSION="20.04.1 LTS (Focal Fossa)"\nID=ubuntu\nID_LIKE=debian\nPRETTY_NAME="Ubuntu 20.04.1 LTS"\nVERSION_ID="20.04"\nHOME_URL="https://www.ubuntu.com/"\nSUPPORT_URL="https://help.ubuntu.com/"\nBUG_REPORT_URL="https://bugs.launchpad.net/ubuntu/"\nPRIVACY_POLICY_URL="https://www.ubuntu.com/legal/terms-and-policies/privacy-policy"\nVERSION_CODENAME=focal\nUBUNTU_CODENAME=focal\n'

TEST_OS_RELEASE = '\n# test data\nID_LIKE="egg spam viking"\nEMPTY=\n# comments and empty lines are ignored\n\nSINGLE_QUOTE=\'single\'\nEMPTY_SINGLE=\'\'\nDOUBLE_QUOTE="double"\nEMPTY_DOUBLE=""\nQUOTES="double\\\'s"\nSPECIALS="\\$\\`\\\\\\\'\\""\n# invalid lines\n=invalid\n=\nINVALID\nIN-VALID=value\nIN VALID=value\n'


# --- test body ---
self_save_version = sys.version
self_save_git = sys._git
self_save_platform = sys.platform
res = platform.release()
print("PlatformTest::test_release: ok")
"###);
    assert_output(&out, r###"PlatformTest::test_release: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/platform/platform_test__test_system.py`.
#[test]
fn test_gen_behavior_std_libs_platform_platform_test__test_system() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "platform_test__test_system"
# subject = "cpython.test_platform.PlatformTest.test_system"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_platform.py::PlatformTest::test_system
"""Auto-ported test: PlatformTest::test_system (CPython 3.12 oracle)."""


import os
import copy
import pickle
import platform
import subprocess
import sys
import unittest
from unittest import mock
from test import support
from test.support import os_helper


FEDORA_OS_RELEASE = 'NAME=Fedora\nVERSION="32 (Thirty Two)"\nID=fedora\nVERSION_ID=32\nVERSION_CODENAME=""\nPLATFORM_ID="platform:f32"\nPRETTY_NAME="Fedora 32 (Thirty Two)"\nANSI_COLOR="0;34"\nLOGO=fedora-logo-icon\nCPE_NAME="cpe:/o:fedoraproject:fedora:32"\nHOME_URL="https://fedoraproject.org/"\nDOCUMENTATION_URL="https://docs.fedoraproject.org/en-US/fedora/f32/system-administrators-guide/"\nSUPPORT_URL="https://fedoraproject.org/wiki/Communicating_and_getting_help"\nBUG_REPORT_URL="https://bugzilla.redhat.com/"\nREDHAT_BUGZILLA_PRODUCT="Fedora"\nREDHAT_BUGZILLA_PRODUCT_VERSION=32\nREDHAT_SUPPORT_PRODUCT="Fedora"\nREDHAT_SUPPORT_PRODUCT_VERSION=32\nPRIVACY_POLICY_URL="https://fedoraproject.org/wiki/Legal:PrivacyPolicy"\n'

UBUNTU_OS_RELEASE = 'NAME="Ubuntu"\nVERSION="20.04.1 LTS (Focal Fossa)"\nID=ubuntu\nID_LIKE=debian\nPRETTY_NAME="Ubuntu 20.04.1 LTS"\nVERSION_ID="20.04"\nHOME_URL="https://www.ubuntu.com/"\nSUPPORT_URL="https://help.ubuntu.com/"\nBUG_REPORT_URL="https://bugs.launchpad.net/ubuntu/"\nPRIVACY_POLICY_URL="https://www.ubuntu.com/legal/terms-and-policies/privacy-policy"\nVERSION_CODENAME=focal\nUBUNTU_CODENAME=focal\n'

TEST_OS_RELEASE = '\n# test data\nID_LIKE="egg spam viking"\nEMPTY=\n# comments and empty lines are ignored\n\nSINGLE_QUOTE=\'single\'\nEMPTY_SINGLE=\'\'\nDOUBLE_QUOTE="double"\nEMPTY_DOUBLE=""\nQUOTES="double\\\'s"\nSPECIALS="\\$\\`\\\\\\\'\\""\n# invalid lines\n=invalid\n=\nINVALID\nIN-VALID=value\nIN VALID=value\n'


# --- test body ---
self_save_version = sys.version
self_save_git = sys._git
self_save_platform = sys.platform
res = platform.system()
print("PlatformTest::test_system: ok")
"###);
    assert_output(&out, r###"PlatformTest::test_system: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/platform/platform_test__test_system_alias.py`.
#[test]
fn test_gen_behavior_std_libs_platform_platform_test__test_system_alias() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "platform_test__test_system_alias"
# subject = "cpython.test_platform.PlatformTest.test_system_alias"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_platform.py::PlatformTest::test_system_alias
"""Auto-ported test: PlatformTest::test_system_alias (CPython 3.12 oracle)."""


import os
import copy
import pickle
import platform
import subprocess
import sys
import unittest
from unittest import mock
from test import support
from test.support import os_helper


FEDORA_OS_RELEASE = 'NAME=Fedora\nVERSION="32 (Thirty Two)"\nID=fedora\nVERSION_ID=32\nVERSION_CODENAME=""\nPLATFORM_ID="platform:f32"\nPRETTY_NAME="Fedora 32 (Thirty Two)"\nANSI_COLOR="0;34"\nLOGO=fedora-logo-icon\nCPE_NAME="cpe:/o:fedoraproject:fedora:32"\nHOME_URL="https://fedoraproject.org/"\nDOCUMENTATION_URL="https://docs.fedoraproject.org/en-US/fedora/f32/system-administrators-guide/"\nSUPPORT_URL="https://fedoraproject.org/wiki/Communicating_and_getting_help"\nBUG_REPORT_URL="https://bugzilla.redhat.com/"\nREDHAT_BUGZILLA_PRODUCT="Fedora"\nREDHAT_BUGZILLA_PRODUCT_VERSION=32\nREDHAT_SUPPORT_PRODUCT="Fedora"\nREDHAT_SUPPORT_PRODUCT_VERSION=32\nPRIVACY_POLICY_URL="https://fedoraproject.org/wiki/Legal:PrivacyPolicy"\n'

UBUNTU_OS_RELEASE = 'NAME="Ubuntu"\nVERSION="20.04.1 LTS (Focal Fossa)"\nID=ubuntu\nID_LIKE=debian\nPRETTY_NAME="Ubuntu 20.04.1 LTS"\nVERSION_ID="20.04"\nHOME_URL="https://www.ubuntu.com/"\nSUPPORT_URL="https://help.ubuntu.com/"\nBUG_REPORT_URL="https://bugs.launchpad.net/ubuntu/"\nPRIVACY_POLICY_URL="https://www.ubuntu.com/legal/terms-and-policies/privacy-policy"\nVERSION_CODENAME=focal\nUBUNTU_CODENAME=focal\n'

TEST_OS_RELEASE = '\n# test data\nID_LIKE="egg spam viking"\nEMPTY=\n# comments and empty lines are ignored\n\nSINGLE_QUOTE=\'single\'\nEMPTY_SINGLE=\'\'\nDOUBLE_QUOTE="double"\nEMPTY_DOUBLE=""\nQUOTES="double\\\'s"\nSPECIALS="\\$\\`\\\\\\\'\\""\n# invalid lines\n=invalid\n=\nINVALID\nIN-VALID=value\nIN VALID=value\n'


# --- test body ---
self_save_version = sys.version
self_save_git = sys._git
self_save_platform = sys.platform
res = platform.system_alias(platform.system(), platform.release(), platform.version())
print("PlatformTest::test_system_alias: ok")
"###);
    assert_output(&out, r###"PlatformTest::test_system_alias: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/platform/platform_test__test_uname_copy.py`.
#[test]
fn test_gen_behavior_std_libs_platform_platform_test__test_uname_copy() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "platform_test__test_uname_copy"
# subject = "cpython.test_platform.PlatformTest.test_uname_copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_platform.py::PlatformTest::test_uname_copy
"""Auto-ported test: PlatformTest::test_uname_copy (CPython 3.12 oracle)."""


import os
import copy
import pickle
import platform
import subprocess
import sys
import unittest
from unittest import mock
from test import support
from test.support import os_helper


FEDORA_OS_RELEASE = 'NAME=Fedora\nVERSION="32 (Thirty Two)"\nID=fedora\nVERSION_ID=32\nVERSION_CODENAME=""\nPLATFORM_ID="platform:f32"\nPRETTY_NAME="Fedora 32 (Thirty Two)"\nANSI_COLOR="0;34"\nLOGO=fedora-logo-icon\nCPE_NAME="cpe:/o:fedoraproject:fedora:32"\nHOME_URL="https://fedoraproject.org/"\nDOCUMENTATION_URL="https://docs.fedoraproject.org/en-US/fedora/f32/system-administrators-guide/"\nSUPPORT_URL="https://fedoraproject.org/wiki/Communicating_and_getting_help"\nBUG_REPORT_URL="https://bugzilla.redhat.com/"\nREDHAT_BUGZILLA_PRODUCT="Fedora"\nREDHAT_BUGZILLA_PRODUCT_VERSION=32\nREDHAT_SUPPORT_PRODUCT="Fedora"\nREDHAT_SUPPORT_PRODUCT_VERSION=32\nPRIVACY_POLICY_URL="https://fedoraproject.org/wiki/Legal:PrivacyPolicy"\n'

UBUNTU_OS_RELEASE = 'NAME="Ubuntu"\nVERSION="20.04.1 LTS (Focal Fossa)"\nID=ubuntu\nID_LIKE=debian\nPRETTY_NAME="Ubuntu 20.04.1 LTS"\nVERSION_ID="20.04"\nHOME_URL="https://www.ubuntu.com/"\nSUPPORT_URL="https://help.ubuntu.com/"\nBUG_REPORT_URL="https://bugs.launchpad.net/ubuntu/"\nPRIVACY_POLICY_URL="https://www.ubuntu.com/legal/terms-and-policies/privacy-policy"\nVERSION_CODENAME=focal\nUBUNTU_CODENAME=focal\n'

TEST_OS_RELEASE = '\n# test data\nID_LIKE="egg spam viking"\nEMPTY=\n# comments and empty lines are ignored\n\nSINGLE_QUOTE=\'single\'\nEMPTY_SINGLE=\'\'\nDOUBLE_QUOTE="double"\nEMPTY_DOUBLE=""\nQUOTES="double\\\'s"\nSPECIALS="\\$\\`\\\\\\\'\\""\n# invalid lines\n=invalid\n=\nINVALID\nIN-VALID=value\nIN VALID=value\n'


# --- test body ---
self_save_version = sys.version
self_save_git = sys._git
self_save_platform = sys.platform
uname = platform.uname()

assert copy.copy(uname) == uname

assert copy.deepcopy(uname) == uname
print("PlatformTest::test_uname_copy: ok")
"###);
    assert_output(&out, r###"PlatformTest::test_uname_copy: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/platform/platform_test__test_uname_pickle.py`.
#[test]
fn test_gen_behavior_std_libs_platform_platform_test__test_uname_pickle() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "platform_test__test_uname_pickle"
# subject = "cpython.test_platform.PlatformTest.test_uname_pickle"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_platform.py::PlatformTest::test_uname_pickle
"""Auto-ported test: PlatformTest::test_uname_pickle (CPython 3.12 oracle)."""


import os
import copy
import pickle
import platform
import subprocess
import sys
import unittest
from unittest import mock
from test import support
from test.support import os_helper


FEDORA_OS_RELEASE = 'NAME=Fedora\nVERSION="32 (Thirty Two)"\nID=fedora\nVERSION_ID=32\nVERSION_CODENAME=""\nPLATFORM_ID="platform:f32"\nPRETTY_NAME="Fedora 32 (Thirty Two)"\nANSI_COLOR="0;34"\nLOGO=fedora-logo-icon\nCPE_NAME="cpe:/o:fedoraproject:fedora:32"\nHOME_URL="https://fedoraproject.org/"\nDOCUMENTATION_URL="https://docs.fedoraproject.org/en-US/fedora/f32/system-administrators-guide/"\nSUPPORT_URL="https://fedoraproject.org/wiki/Communicating_and_getting_help"\nBUG_REPORT_URL="https://bugzilla.redhat.com/"\nREDHAT_BUGZILLA_PRODUCT="Fedora"\nREDHAT_BUGZILLA_PRODUCT_VERSION=32\nREDHAT_SUPPORT_PRODUCT="Fedora"\nREDHAT_SUPPORT_PRODUCT_VERSION=32\nPRIVACY_POLICY_URL="https://fedoraproject.org/wiki/Legal:PrivacyPolicy"\n'

UBUNTU_OS_RELEASE = 'NAME="Ubuntu"\nVERSION="20.04.1 LTS (Focal Fossa)"\nID=ubuntu\nID_LIKE=debian\nPRETTY_NAME="Ubuntu 20.04.1 LTS"\nVERSION_ID="20.04"\nHOME_URL="https://www.ubuntu.com/"\nSUPPORT_URL="https://help.ubuntu.com/"\nBUG_REPORT_URL="https://bugs.launchpad.net/ubuntu/"\nPRIVACY_POLICY_URL="https://www.ubuntu.com/legal/terms-and-policies/privacy-policy"\nVERSION_CODENAME=focal\nUBUNTU_CODENAME=focal\n'

TEST_OS_RELEASE = '\n# test data\nID_LIKE="egg spam viking"\nEMPTY=\n# comments and empty lines are ignored\n\nSINGLE_QUOTE=\'single\'\nEMPTY_SINGLE=\'\'\nDOUBLE_QUOTE="double"\nEMPTY_DOUBLE=""\nQUOTES="double\\\'s"\nSPECIALS="\\$\\`\\\\\\\'\\""\n# invalid lines\n=invalid\n=\nINVALID\nIN-VALID=value\nIN VALID=value\n'


# --- test body ---
def clear_caches():
    platform._platform_cache.clear()
    platform._sys_version_cache.clear()
    platform._uname_cache = None
    platform._os_release_cache = None
self_save_version = sys.version
self_save_git = sys._git
self_save_platform = sys.platform
orig = platform.uname()
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    pickled = pickle.dumps(orig, proto)
    restored = pickle.loads(pickled)

    assert restored == orig
print("PlatformTest::test_uname_pickle: ok")
"###);
    assert_output(&out, r###"PlatformTest::test_uname_pickle: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/platform/platform_test__test_version.py`.
#[test]
fn test_gen_behavior_std_libs_platform_platform_test__test_version() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "platform_test__test_version"
# subject = "cpython.test_platform.PlatformTest.test_version"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_platform.py::PlatformTest::test_version
"""Auto-ported test: PlatformTest::test_version (CPython 3.12 oracle)."""


import os
import copy
import pickle
import platform
import subprocess
import sys
import unittest
from unittest import mock
from test import support
from test.support import os_helper


FEDORA_OS_RELEASE = 'NAME=Fedora\nVERSION="32 (Thirty Two)"\nID=fedora\nVERSION_ID=32\nVERSION_CODENAME=""\nPLATFORM_ID="platform:f32"\nPRETTY_NAME="Fedora 32 (Thirty Two)"\nANSI_COLOR="0;34"\nLOGO=fedora-logo-icon\nCPE_NAME="cpe:/o:fedoraproject:fedora:32"\nHOME_URL="https://fedoraproject.org/"\nDOCUMENTATION_URL="https://docs.fedoraproject.org/en-US/fedora/f32/system-administrators-guide/"\nSUPPORT_URL="https://fedoraproject.org/wiki/Communicating_and_getting_help"\nBUG_REPORT_URL="https://bugzilla.redhat.com/"\nREDHAT_BUGZILLA_PRODUCT="Fedora"\nREDHAT_BUGZILLA_PRODUCT_VERSION=32\nREDHAT_SUPPORT_PRODUCT="Fedora"\nREDHAT_SUPPORT_PRODUCT_VERSION=32\nPRIVACY_POLICY_URL="https://fedoraproject.org/wiki/Legal:PrivacyPolicy"\n'

UBUNTU_OS_RELEASE = 'NAME="Ubuntu"\nVERSION="20.04.1 LTS (Focal Fossa)"\nID=ubuntu\nID_LIKE=debian\nPRETTY_NAME="Ubuntu 20.04.1 LTS"\nVERSION_ID="20.04"\nHOME_URL="https://www.ubuntu.com/"\nSUPPORT_URL="https://help.ubuntu.com/"\nBUG_REPORT_URL="https://bugs.launchpad.net/ubuntu/"\nPRIVACY_POLICY_URL="https://www.ubuntu.com/legal/terms-and-policies/privacy-policy"\nVERSION_CODENAME=focal\nUBUNTU_CODENAME=focal\n'

TEST_OS_RELEASE = '\n# test data\nID_LIKE="egg spam viking"\nEMPTY=\n# comments and empty lines are ignored\n\nSINGLE_QUOTE=\'single\'\nEMPTY_SINGLE=\'\'\nDOUBLE_QUOTE="double"\nEMPTY_DOUBLE=""\nQUOTES="double\\\'s"\nSPECIALS="\\$\\`\\\\\\\'\\""\n# invalid lines\n=invalid\n=\nINVALID\nIN-VALID=value\nIN VALID=value\n'


# --- test body ---
self_save_version = sys.version
self_save_git = sys._git
self_save_platform = sys.platform
res = platform.version()
print("PlatformTest::test_version: ok")
"###);
    assert_output(&out, r###"PlatformTest::test_version: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/platform/processor_returns_str.py`.
#[test]
fn test_gen_behavior_std_libs_platform_processor_returns_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "processor_returns_str"
# subject = "platform.processor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform.processor: processor() returns a str (may be empty on some hosts, only type asserted)"""
import platform

out = platform.processor()
assert type(out).__name__ == "str", "processor() returns str"

print("processor_returns_str OK")
"###);
    assert_output(&out, r###"processor_returns_str OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/platform/python_implementation_is_cpython.py`.
#[test]
fn test_gen_behavior_std_libs_platform_python_implementation_is_cpython() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "python_implementation_is_cpython"
# subject = "platform.python_implementation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform.python_implementation: python_implementation() returns 'CPython' on the reference interpreter"""
import platform

assert platform.python_implementation() == "CPython", "running CPython"

print("python_implementation_is_cpython OK")
"###);
    assert_output(&out, r###"python_implementation_is_cpython OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/platform/python_version_dotted_str.py`.
#[test]
fn test_gen_behavior_std_libs_platform_python_version_dotted_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "python_version_dotted_str"
# subject = "platform.python_version"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform.python_version: python_version() returns a dotted version str that starts with '3.' under CPython 3.12"""
import platform

out = platform.python_version()
assert type(out).__name__ == "str", "python_version() returns str"
assert out.startswith("3."), "python_version() starts with '3.'"

print("python_version_dotted_str OK")
"###);
    assert_output(&out, r###"python_version_dotted_str OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/platform/python_version_tuple_joins_to_version.py`.
#[test]
fn test_gen_behavior_std_libs_platform_python_version_tuple_joins_to_version() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "python_version_tuple_joins_to_version"
# subject = "platform.python_version_tuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform.python_version_tuple: python_version_tuple() yields a 3-part tuple of digit strings whose '.'-join equals python_version()"""
import platform

parts = platform.python_version_tuple()
assert len(parts) == 3, "version tuple has 3 parts"
assert ".".join(parts) == platform.python_version(), "tuple joins to version"
assert all(p.isdigit() for p in parts[:2]), "major/minor are digits"

print("python_version_tuple_joins_to_version OK")
"###);
    assert_output(&out, r###"python_version_tuple_joins_to_version OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/platform/release_returns_str.py`.
#[test]
fn test_gen_behavior_std_libs_platform_release_returns_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "release_returns_str"
# subject = "platform.release"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform.release: release() returns a str OS release (value host-dependent, only type asserted)"""
import platform

out = platform.release()
assert type(out).__name__ == "str", "release() returns str"

print("release_returns_str OK")
"###);
    assert_output(&out, r###"release_returns_str OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/platform/sys_version_banner_parsing.py`.
#[test]
fn test_gen_behavior_std_libs_platform_sys_version_banner_parsing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "sys_version_banner_parsing"
# subject = "platform._sys_version"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform._sys_version: _sys_version parses an interpreter banner into (name, version, branch, revision, buildno, builddate, compiler), applying the build-date truncation rules and the compiler suffix"""
import platform

CASES = (
    (
        "2.4.3 (#1, Jun 21 2006, 13:54:21) \n[GCC 3.3.4 (pre 3.3.5 20040809)]",
        ("CPython", "2.4.3", "1", "Jun 21 2006 13:54:21",
         "GCC 3.3.4 (pre 3.3.5 20040809)"),
    ),
    (
        "2.4.3 (truncation, date, t) \n[GCC]",
        ("CPython", "2.4.3", "truncation", "date t", "GCC"),
    ),
    (
        "2.4.3 (truncation, date, ) \n[GCC]",
        ("CPython", "2.4.3", "truncation", "date", "GCC"),
    ),
    (
        "2.4.3 (truncation) \n[GCC]",
        ("CPython", "2.4.3", "truncation", "", "GCC"),
    ),
)
for banner, expected in CASES:
    name, version, branch, revision, buildno, builddate, compiler = \
        platform._sys_version(banner)
    got = (name, version, buildno, builddate, compiler)
    assert got == expected, f"parse {banner!r} -> {got!r}"
    assert branch == "", "branch blank without scm tag"
    assert revision == "", "revision blank without scm tag"

print("sys_version_banner_parsing OK")
"###);
    assert_output(&out, r###"sys_version_banner_parsing OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/platform/system_returns_nonempty_str.py`.
#[test]
fn test_gen_behavior_std_libs_platform_system_returns_nonempty_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "system_returns_nonempty_str"
# subject = "platform.system"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform.system: system() returns a non-empty str naming the OS (value is host-dependent, so only shape is asserted)"""
import platform

out = platform.system()
assert type(out).__name__ == "str", "system() returns str"
assert len(out) > 0, "system() is non-empty"

print("system_returns_nonempty_str OK")
"###);
    assert_output(&out, r###"system_returns_nonempty_str OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/platform/uname_namedtuple_contract.py`.
#[test]
fn test_gen_behavior_std_libs_platform_uname_namedtuple_contract() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "uname_namedtuple_contract"
# subject = "platform.uname"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform.uname: uname() returns a 6-field named tuple: positional/negative/attribute access agree, _fields/_asdict/_replace behave, and the value round-trips through tuple()/slice/copy"""
import platform

import copy

res = platform.uname()

# Six positional fields, each reachable by name and by index (incl. negatives).
assert len(res) == 6, "uname has 6 fields"
assert res[0] == res.system and res[-6] == res.system, "system at 0 / -6"
assert res[1] == res.node and res[-5] == res.node, "node at 1 / -5"
assert res[2] == res.release and res[-4] == res.release, "release at 2 / -4"
assert res[3] == res.version and res[-3] == res.version, "version at 3 / -3"
assert res[4] == res.machine and res[-2] == res.machine, "machine at 4 / -2"
assert res[5] == res.processor and res[-1] == res.processor, "processor at 5/-1"

# Field names and casting to a plain tuple.
assert res._fields == (
    "system", "node", "release", "version", "machine", "processor"
), "field names"
expected = (res.system, res.node, res.release,
            res.version, res.machine, res.processor)
assert tuple(res) == expected, "tuple() yields the 6 values in order"
assert res[:] == expected, "full slice equals tuple"
assert res[:5] == expected[:5], "partial slice"

# _asdict preserves order and keys.
d = res._asdict()
assert len(d) == 6 and "processor" in d, "asdict has 6 keys incl processor"
assert list(d.values()) == list(expected), "asdict values match"

# _replace overrides named fields and leaves the rest untouched.
new = res._replace(system="S", node="N", release="R",
                   version="V", machine="M")
assert (new.system, new.node, new.release, new.version, new.machine) == \
    ("S", "N", "R", "V", "M"), "replaced fields"
assert new.processor == res.processor, "unreplaced field preserved"

# copy / deepcopy compare equal to the original.
assert copy.copy(res) == res, "shallow copy equal"
assert copy.deepcopy(res) == res, "deep copy equal"

print("uname_namedtuple_contract OK")
"###);
    assert_output(&out, r###"uname_namedtuple_contract OK
"###);
}
