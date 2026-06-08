# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "aifc"
# dimension = "behavior"
# case = "aifc_misc_test__test_write_markers_values"
# subject = "cpython.test_aifc.AifcMiscTest.test_write_markers_values"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_aifc.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_aifc.py::AifcMiscTest::test_write_markers_values
"""Auto-ported test: AifcMiscTest::test_write_markers_values (CPython 3.12 oracle)."""


from test.support import findfile
from test.support.os_helper import TESTFN, unlink
from test.support.warnings_helper import check_no_resource_warning, import_deprecated
import unittest
from unittest import mock
from test import audiotests
import io
import sys
import struct


aifc = import_deprecated('aifc')

audioop = import_deprecated('audioop')

class AifcTest(audiotests.AudioWriteTests, audiotests.AudioTestsWithSourceFile):
    module = aifc
    close_fd = True
    test_unseekable_read = None

class AifcPCM8Test(AifcTest, unittest.TestCase):
    sndfilename = 'pluck-pcm8.aiff'
    sndfilenframes = 3307
    nchannels = 2
    sampwidth = 1
    framerate = 11025
    nframes = 48
    comptype = b'NONE'
    compname = b'not compressed'
    frames = bytes.fromhex('      02FF 4B00 3104 8008 CB06 4803 BF01 03FE B8FA B4F3 29EB 1AE6       EDE4 C6E2 0EE0 EFE0 57E2 FBE8 13EF D8F7 97FB F5FC 08FB DFFB       11FA 3EFB BCFC 66FF CF04 4309 C10E 5112 EE17 8216 7F14 8012       490E 520D EF0F CE0F E40C 630A 080A 2B0B 510E 8B11 B60E 440A       ')

class AifcPCM16Test(AifcTest, unittest.TestCase):
    sndfilename = 'pluck-pcm16.aiff'
    sndfilenframes = 3307
    nchannels = 2
    sampwidth = 2
    framerate = 11025
    nframes = 48
    comptype = b'NONE'
    compname = b'not compressed'
    frames = bytes.fromhex('      022EFFEA 4B5D00F6 311804EA 80E10840 CBE106B1 48A903F5 BFE601B2 036CFE7B       B858FA3E B4B1F34F 299AEBCA 1A5DE6DA EDFAE491 C628E275 0E09E0B5 EF2AE029       5758E271 FB35E83F 1376EF86 D82BF727 9790FB76 F5FAFC0F 0867FB9C DF30FB43       117EFA36 3EE5FB5B BC79FCB1 66D9FF5D CF150412 431D097C C1BA0EC8 512112A1       EEE21753 82071665 7FFF1443 8004128F 49A20EAF 52BB0DBA EFB40F60 CE3C0FBF       E4B30CEC 63430A5C 08C80A20 2BBB0B08 514A0E43 8BCF1139 B6F60EEB 44120A5E       ')

class AifcPCM24Test(AifcTest, unittest.TestCase):
    sndfilename = 'pluck-pcm24.aiff'
    sndfilenframes = 3307
    nchannels = 2
    sampwidth = 3
    framerate = 11025
    nframes = 48
    comptype = b'NONE'
    compname = b'not compressed'
    frames = bytes.fromhex('      022D65FFEB9D 4B5A0F00FA54 3113C304EE2B 80DCD6084303       CBDEC006B261 48A99803F2F8 BFE82401B07D 036BFBFE7B5D       B85756FA3EC9 B4B055F3502B 299830EBCB62 1A5CA7E6D99A       EDFA3EE491BD C625EBE27884 0E05A9E0B6CF EF2929E02922       5758D8E27067 FB3557E83E16 1377BFEF8402 D82C5BF7272A       978F16FB7745 F5F865FC1013 086635FB9C4E DF30FCFB40EE       117FE0FA3438 3EE6B8FB5AC3 BC77A3FCB2F4 66D6DAFF5F32       CF13B9041275 431D69097A8C C1BB600EC74E 5120B912A2BA       EEDF641754C0 8207001664B7 7FFFFF14453F 8000001294E6       499C1B0EB3B2 52B73E0DBCA0 EFB2B20F5FD8 CE3CDB0FBE12       E4B49C0CEA2D 6344A80A5A7C 08C8FE0A1FFE 2BB9860B0A0E       51486F0E44E1 8BCC64113B05 B6F4EC0EEB36 4413170A5B48       ')

class AifcPCM32Test(AifcTest, unittest.TestCase):
    sndfilename = 'pluck-pcm32.aiff'
    sndfilenframes = 3307
    nchannels = 2
    sampwidth = 4
    framerate = 11025
    nframes = 48
    comptype = b'NONE'
    compname = b'not compressed'
    frames = bytes.fromhex('      022D65BCFFEB9D92 4B5A0F8000FA549C 3113C34004EE2BC0 80DCD680084303E0       CBDEC0C006B26140 48A9980003F2F8FC BFE8248001B07D92 036BFB60FE7B5D34       B8575600FA3EC920 B4B05500F3502BC0 29983000EBCB6240 1A5CA7A0E6D99A60       EDFA3E80E491BD40 C625EB80E27884A0 0E05A9A0E0B6CFE0 EF292940E0292280       5758D800E2706700 FB3557D8E83E1640 1377BF00EF840280 D82C5B80F7272A80       978F1600FB774560 F5F86510FC101364 086635A0FB9C4E20 DF30FC40FB40EE28       117FE0A0FA3438B0 3EE6B840FB5AC3F0 BC77A380FCB2F454 66D6DA80FF5F32B4       CF13B980041275B0 431D6980097A8C00 C1BB60000EC74E00 5120B98012A2BAA0       EEDF64C01754C060 820700001664B780 7FFFFFFF14453F40 800000001294E6E0       499C1B000EB3B270 52B73E000DBCA020 EFB2B2E00F5FD880 CE3CDB400FBE1270       E4B49CC00CEA2D90 6344A8800A5A7CA0 08C8FE800A1FFEE0 2BB986C00B0A0E00       51486F800E44E190 8BCC6480113B0580 B6F4EC000EEB3630 441317800A5B48A0       ')

class AifcULAWTest(AifcTest, unittest.TestCase):
    sndfilename = 'pluck-ulaw.aifc'
    sndfilenframes = 3307
    nchannels = 2
    sampwidth = 2
    framerate = 11025
    nframes = 48
    comptype = b'ulaw'
    compname = b''
    frames = bytes.fromhex('      022CFFE8 497C0104 307C04DC 8284083C CB84069C 497C03DC BE8401AC 036CFE74       B684FA24 B684F344 2A7CEC04 19FCE704 EE04E504 C584E204 0E3CE104 EF04DF84       557CE204 FB24E804 12FCEF04 D784F744 9684FB64 F5C4FC24 083CFBA4 DF84FB24       11FCFA24 3E7CFB64 BA84FCB4 657CFF5C CF84041C 417C093C C1840EBC 517C12FC       EF0416FC 828415FC 7D7C13FC 828412FC 497C0EBC 517C0DBC F0040F3C CD840FFC       E5040CBC 617C0A3C 08BC0A3C 2C7C0B3C 517C0E3C 8A8410FC B6840EBC 457C0A3C       ')
    if sys.byteorder != 'big':
        frames = audioop.byteswap(frames, 2)

class AifcALAWTest(AifcTest, unittest.TestCase):
    sndfilename = 'pluck-alaw.aifc'
    sndfilenframes = 3307
    nchannels = 2
    sampwidth = 2
    framerate = 11025
    nframes = 48
    comptype = b'alaw'
    compname = b''
    frames = bytes.fromhex('      0230FFE8 4A0000F8 310004E0 82000840 CB0006A0 4A0003F0 BE0001A8 0370FE78       BA00FA20 B600F340 2900EB80 1A80E680 ED80E480 C700E280 0E40E080 EF80E080       5600E280 FB20E880 1380EF80 D900F740 9600FB60 F5C0FC10 0840FBA0 DF00FB20       1180FA20 3F00FB60 BE00FCB0 6600FF58 CF000420 42000940 C1000EC0 52001280       EE801780 82001680 7E001480 82001280 4A000EC0 52000DC0 EF800F40 CF000FC0       E4800CC0 62000A40 08C00A40 2B000B40 52000E40 8A001180 B6000EC0 46000A40       ')
    if sys.byteorder != 'big':
        frames = audioop.byteswap(frames, 2)


# --- test body ---
fout = aifc.open(io.BytesIO(), 'wb')

assert fout.getmarkers() == None
fout.setmark(1, 0, b'foo1')
fout.setmark(1, 1, b'foo2')

assert fout.getmark(1) == (1, 1, b'foo2')

assert fout.getmarkers() == [(1, 1, b'foo2')]
fout.initfp(None)
print("AifcMiscTest::test_write_markers_values: ok")
