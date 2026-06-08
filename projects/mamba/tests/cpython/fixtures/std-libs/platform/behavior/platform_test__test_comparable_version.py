# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "platform_test__test_comparable_version"
# subject = "cpython.test_platform.PlatformTest.test__comparable_version"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_platform.py::PlatformTest::test__comparable_version
"""Auto-ported test: PlatformTest::test__comparable_version (CPython 3.12 oracle)."""


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
from platform import _comparable_version as V

assert V('1.2.3') == V('1.2.3')

assert V('1.2.3') < V('1.2.10')

assert V('1.2.3.4') == V('1_2-3+4')

assert V('1.2spam') < V('1.2dev')

assert V('1.2dev') < V('1.2alpha')

assert V('1.2dev') < V('1.2a')

assert V('1.2alpha') < V('1.2beta')

assert V('1.2a') < V('1.2b')

assert V('1.2beta') < V('1.2c')

assert V('1.2b') < V('1.2c')

assert V('1.2c') < V('1.2RC')

assert V('1.2c') < V('1.2rc')

assert V('1.2RC') < V('1.2.0')

assert V('1.2rc') < V('1.2.0')

assert V('1.2.0') < V('1.2pl')

assert V('1.2.0') < V('1.2p')

assert V('1.5.1') < V('1.5.2b2')

assert V('3.10a') < V('161')

assert V('8.02') == V('8.02')

assert V('3.4j') < V('1996.07.12')

assert V('3.1.1.6') < V('3.2.pl0')

assert V('2g6') < V('11g')

assert V('0.9') < V('2.2')

assert V('1.2') < V('1.2.1')

assert V('1.1') < V('1.2.2')

assert V('1.1') < V('1.2')

assert V('1.2.1') < V('1.2.2')

assert V('1.2') < V('1.2.2')

assert V('0.4') < V('0.4.0')

assert V('1.13++') < V('5.5.kw')

assert V('0.960923') < V('2.2beta29')
print("PlatformTest::test__comparable_version: ok")
