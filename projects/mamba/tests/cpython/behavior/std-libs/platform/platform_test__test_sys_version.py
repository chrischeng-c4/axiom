# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "platform_test__test_sys_version"
# subject = "cpython.test_platform.PlatformTest.test_sys_version"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_platform.py::PlatformTest::test_sys_version
"""Auto-ported test: PlatformTest::test_sys_version (CPython 3.12 oracle)."""


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
for input, output in (('2.4.3 (#1, Jun 21 2006, 13:54:21) \n[GCC 3.3.4 (pre 3.3.5 20040809)]', ('CPython', '2.4.3', '', '', '1', 'Jun 21 2006 13:54:21', 'GCC 3.3.4 (pre 3.3.5 20040809)')), ('2.4.3 (truncation, date, t) \n[GCC]', ('CPython', '2.4.3', '', '', 'truncation', 'date t', 'GCC')), ('2.4.3 (truncation, date, ) \n[GCC]', ('CPython', '2.4.3', '', '', 'truncation', 'date', 'GCC')), ('2.4.3 (truncation, date,) \n[GCC]', ('CPython', '2.4.3', '', '', 'truncation', 'date', 'GCC')), ('2.4.3 (truncation, date) \n[GCC]', ('CPython', '2.4.3', '', '', 'truncation', 'date', 'GCC')), ('2.4.3 (truncation, d) \n[GCC]', ('CPython', '2.4.3', '', '', 'truncation', 'd', 'GCC')), ('2.4.3 (truncation, ) \n[GCC]', ('CPython', '2.4.3', '', '', 'truncation', '', 'GCC')), ('2.4.3 (truncation,) \n[GCC]', ('CPython', '2.4.3', '', '', 'truncation', '', 'GCC')), ('2.4.3 (truncation) \n[GCC]', ('CPython', '2.4.3', '', '', 'truncation', '', 'GCC'))):
    name, version, branch, revision, buildno, builddate, compiler = platform._sys_version(input)

    assert (name, version, '', '', buildno, builddate, compiler) == output
sys_versions = {('2.6.1 (r261:67515, Dec  6 2008, 15:26:00) \n[GCC 4.0.1 (Apple Computer, Inc. build 5370)]', ('CPython', 'tags/r261', '67515'), self_save_platform): ('CPython', '2.6.1', 'tags/r261', '67515', ('r261:67515', 'Dec  6 2008 15:26:00'), 'GCC 4.0.1 (Apple Computer, Inc. build 5370)'), ('3.10.8 (tags/v3.10.8:aaaf517424, Feb 14 2023, 16:28:12) [GCC 9.4.0]', None, 'linux'): ('CPython', '3.10.8', '', '', ('tags/v3.10.8:aaaf517424', 'Feb 14 2023 16:28:12'), 'GCC 9.4.0'), ('2.5 (trunk:6107, Mar 26 2009, 13:02:18) \n[Java HotSpot(TM) Client VM ("Apple Computer, Inc.")]', ('Jython', 'trunk', '6107'), 'java1.5.0_16'): ('Jython', '2.5.0', 'trunk', '6107', ('trunk:6107', 'Mar 26 2009'), 'java1.5.0_16'), ('2.5.2 (63378, Mar 26 2009, 18:03:29)\n[PyPy 1.0.0]', ('PyPy', 'trunk', '63378'), self_save_platform): ('PyPy', '2.5.2', 'trunk', '63378', ('63378', 'Mar 26 2009'), '')}
for (version_tag, scm, sys_platform), info in sys_versions.items():
    sys.version = version_tag
    if scm is None:
        if hasattr(sys, '_git'):
            del sys._git
    else:
        sys._git = scm
    if sys_platform is not None:
        sys.platform = sys_platform

    assert platform.python_implementation() == info[0]

    assert platform.python_version() == info[1]

    assert platform.python_branch() == info[2]

    assert platform.python_revision() == info[3]

    assert platform.python_build() == info[4]

    assert platform.python_compiler() == info[5]
try:
    platform._sys_version('2. 4.3 (truncation) \n[GCC]')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("PlatformTest::test_sys_version: ok")
