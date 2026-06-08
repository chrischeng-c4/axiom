# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "platform_test__test_mac_ver"
# subject = "cpython.test_platform.PlatformTest.test_mac_ver"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_platform.py::PlatformTest::test_mac_ver
"""Auto-ported test: PlatformTest::test_mac_ver (CPython 3.12 oracle)."""


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
res = platform.mac_ver()
if platform.uname().system == 'Darwin':
    output = subprocess.check_output(['sw_vers'], text=True)
    for line in output.splitlines():
        if line.startswith('ProductVersion:'):
            real_ver = line.strip().split()[-1]
            break
    else:

        raise AssertionError(f'failed to parse sw_vers output: {output!r}')
    result_list = res[0].split('.')
    expect_list = real_ver.split('.')
    len_diff = len(result_list) - len(expect_list)
    if len_diff > 0:
        expect_list.extend(['0'] * len_diff)
    if result_list != ['10', '16']:

        assert result_list == expect_list

    assert res[1] == ('', '', '')
    if sys.byteorder == 'little':

        assert res[2] in ('i386', 'x86_64', 'arm64')
    else:

        assert res[2] == 'PowerPC'
print("PlatformTest::test_mac_ver: ok")
