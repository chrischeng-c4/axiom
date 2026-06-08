# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "platform_test__test_parse_os_release"
# subject = "cpython.test_platform.PlatformTest.test_parse_os_release"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_platform.py::PlatformTest::test_parse_os_release
"""Auto-ported test: PlatformTest::test_parse_os_release (CPython 3.12 oracle)."""


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
info = platform._parse_os_release(FEDORA_OS_RELEASE.splitlines())

assert info['NAME'] == 'Fedora'

assert info['ID'] == 'fedora'

assert 'ID_LIKE' not in info

assert info['VERSION_CODENAME'] == ''
info = platform._parse_os_release(UBUNTU_OS_RELEASE.splitlines())

assert info['NAME'] == 'Ubuntu'

assert info['ID'] == 'ubuntu'

assert info['ID_LIKE'] == 'debian'

assert info['VERSION_CODENAME'] == 'focal'
info = platform._parse_os_release(TEST_OS_RELEASE.splitlines())
expected = {'ID': 'linux', 'NAME': 'Linux', 'PRETTY_NAME': 'Linux', 'ID_LIKE': 'egg spam viking', 'EMPTY': '', 'DOUBLE_QUOTE': 'double', 'EMPTY_DOUBLE': '', 'SINGLE_QUOTE': 'single', 'EMPTY_SINGLE': '', 'QUOTES': "double's", 'SPECIALS': '$`\\\'"'}

assert info == expected

assert len(info['SPECIALS']) == 5
print("PlatformTest::test_parse_os_release: ok")
