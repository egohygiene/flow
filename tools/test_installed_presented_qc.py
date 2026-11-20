#!/usr/bin/env python3
"""Independent saved-WAV observation controls, without native execution."""
from pathlib import Path
import re
import struct
import tempfile
import unittest

import check_installed_presented_qc as proof


class Cases(unittest.TestCase):
    def setUp(self):
        self.temp = self.enterContext(tempfile.TemporaryDirectory())
        self.path = Path(self.temp) / "selected.wav"
        self.pcm = struct.pack("<ffff", .25, -.25, .5, -.5)
        self.wave16 = bytes.fromhex("52494646 40000000 57415645 666d7420 10000000 0300 0200 "
            "80bb0000 00dc0500 0800 2000 66616374 04000000 02000000 64617461 10000000") + self.pcm
        self.wave18 = bytes.fromhex("52494646 42000000 57415645 666d7420 12000000 0300 0200 "
            "80bb0000 00dc0500 0800 2000 0000 66616374 04000000 02000000 64617461 10000000") + self.pcm

    def test_exact_sample_bytes_survive_both_native_fmt_sizes(self):
        for raw in [self.wave16, self.wave18]:
            self.path.write_bytes(raw)
            result = proof.float_wav(self.path)
            self.assertEqual(result, (raw, self.pcm, 2, 2))

    def test_fact_precision_nan_and_truncation_cannot_be_equivalence_evidence(self):
        wrong_fact = bytearray(self.wave16); wrong_fact[44:48] = (1).to_bytes(4, "little")
        wrong_bits = bytearray(self.wave16); wrong_bits[34:36] = (16).to_bytes(2, "little")
        nan = bytearray(self.wave16); nan[56:60] = bytes.fromhex("0000c07f")
        for raw in [wrong_fact, wrong_bits, nan]:
            self.path.write_bytes(raw)
            with self.assertRaisesRegex(ValueError, "^" + re.escape("finite complete native Float32 PCM/clock required") + "$"):
                proof.float_wav(self.path)
        self.path.write_bytes(self.wave16[:-1])
        with self.assertRaisesRegex(ValueError, "^complete bounded RIFF envelope required$"):
            proof.float_wav(self.path)


if __name__ == "__main__": unittest.main()
