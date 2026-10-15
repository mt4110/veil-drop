import os
from pathlib import Path
import runpy
import tempfile
import unittest
import zipfile


class ArchiveTest(unittest.TestCase):
    def test_epoch_license_timestamp_keeps_text_and_executable_mode(self):
        root = Path(tempfile.mkdtemp(prefix='veil-drop-archive-test-'))
        bundle = root / 'bundle'
        bundle.mkdir()
        license_text = bundle / 'LICENSE'
        license_text.write_bytes(b'synthetic license text\n')
        os.utime(license_text, (0, 0))
        binary = bundle / 'veil-drop'
        binary.write_bytes(b'synthetic executable')
        binary.chmod(0o755)
        script = Path(__file__).resolve().parents[1] / 'scripts/package-linux.py'
        archive = root / 'candidate.zip'
        runpy.run_path(str(script))['write_archive'](bundle, archive)
        with zipfile.ZipFile(archive) as z:
            self.assertEqual(z.read('LICENSE'), license_text.read_bytes())
            self.assertEqual(z.getinfo('LICENSE').date_time[0], 1980)
            self.assertEqual(z.read('veil-drop'), binary.read_bytes())
            self.assertEqual((z.getinfo('veil-drop').external_attr >> 16) & 0o777, 0o755)
        # Retain fixtures for diagnostics; do not remove owner files.
