"""Run with python3 -m unittest discover -s tools -p test_session.py."""
from pathlib import Path
import json
import shutil
import subprocess
import tempfile
import tarfile
import unittest


class SessionTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.repo = self.base / 'repo'
        self.repo.mkdir()
        self.run_git('init', '-b', 'main')
        self.run_git('config', 'user.name', 'test')
        self.run_git('config', 'user.email', 'test@localhost')
        (self.repo / 'tools').mkdir()
        shutil.copy(Path(__file__).with_name('session.py'), self.repo / 'tools/session.py')
        (self.repo / 'changed').write_text('before')
        (self.repo / 'deleted').write_text('remove me')
        (self.repo / '.gitignore').write_text('ignored\n')
        self.run_git('add', '.')
        self.run_git('commit', '-m', 'base')

    def run_git(self, *args):
        return subprocess.check_output(['git', '-C', str(self.repo), *args], text=True).strip()

    def session(self, *args, ok=True):
        result = subprocess.run(['python3', str(self.repo / 'tools/session.py'), *args],
                                text=True, capture_output=True)
        self.assertEqual(result.returncode == 0, ok, result.stdout + result.stderr)
        return result

    def test_claim_rejects_same_area_and_checks_owner(self):
        self.session('claim', 'M2-03a', '--owner', 'one')
        self.session('claim', 'M2-05a', '--owner', 'two', ok=False)
        self.session('claim', 'M1-02a', '--owner', 'two')
        self.session('release', 'M2-03a', '--owner', 'two', ok=False)
        self.session('release', 'M2-03a', '--owner', 'one')
        self.session('claim', 'M2-05a', '--owner', 'two')

    def test_claim_shared_between_worktrees(self):
        other = self.base / 'other'
        self.run_git('worktree', 'add', '-b', 'other', str(other))
        self.session('claim', 'M3-04a', '--owner', 'one')
        result = subprocess.run(['python3', str(other / 'tools/session.py'), 'claim',
                                 'M3-05a', '--owner', 'two'], capture_output=True)
        self.assertNotEqual(result.returncode, 0)

    def test_export_preserves_index_and_restores_dirty_files(self):
        (self.repo / 'changed').write_text('staged')
        self.run_git('add', 'changed')
        (self.repo / 'changed').write_text('working')
        (self.repo / 'deleted').unlink()
        (self.repo / 'new').write_text('untracked')
        (self.repo / 'ignored').write_text('excluded')
        before = self.run_git('status', '--porcelain=v1')
        head = self.run_git('rev-parse', 'HEAD')
        index = (self.repo / '.git/index').read_bytes()
        output = self.base / 'transfer'
        reference = self.base / 're'
        for folder in ('decomp', 'notes', 'scripts', 'bench'):
            (reference / folder).mkdir(parents=True)
            (reference / folder / 'fixture').write_text(folder)
        (reference / 'README.md').write_text('reference')
        self.session('export', str(output), '--include-reference')
        self.assertEqual(before, self.run_git('status', '--porcelain=v1'))
        self.assertEqual(head, self.run_git('rev-parse', 'HEAD'))
        self.assertEqual(index, (self.repo / '.git/index').read_bytes())
        manifest = json.loads((output / 'manifest.json').read_text())
        with tarfile.open(output / manifest['reference']) as archive:
            self.assertIn('re/decomp/fixture', archive.getnames())
        clone = self.base / 'clone'
        subprocess.run(['git', 'clone', '-b', manifest['branch'], str(output / 'rsolver.bundle'), str(clone)],
                       check=True, capture_output=True)
        self.assertEqual((clone / 'changed').read_text(), 'working')
        self.assertEqual((clone / 'new').read_text(), 'untracked')
        self.assertFalse((clone / 'deleted').exists())
        self.assertFalse((clone / 'ignored').exists())
        self.assertFalse(self.run_git('for-each-ref', 'refs/heads/handoff-'))
        self.session('export', str(self.repo / 'bad'), ok=False)


if __name__ == '__main__':
    unittest.main()
