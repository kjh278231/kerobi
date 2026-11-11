#!/usr/bin/env python3
"""Merge a finished work branch into main and refresh the generated shared docs under a seconds-long `shared` claim.

  python3 tools/merge_task.py work/m2-06g --owner NAME      # run from the main worktree, clean status
"""
import argparse, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def sh(*cmd, check=True):
    return subprocess.run(cmd, cwd=ROOT, check=check, text=True, capture_output=True)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('branch')
    ap.add_argument('--owner', required=True)
    args = ap.parse_args()
    if sh('git', 'branch', '--show-current').stdout.strip() != 'main' or sh('git', 'status', '--porcelain').stdout.strip():
        sys.exit('main worktree must be on main with a clean status')
    claim = sh('python3', 'tools/session.py', 'claim', 'shared', '--owner', args.owner, check=False)
    if claim.returncode:
        sys.exit('shared is held, retry in a minute: ' + claim.stderr.strip())
    try:
        merged = sh('git', 'merge', '--no-ff', args.branch, '-m', 'Merge ' + args.branch, check=False)
        if merged.returncode:
            sh('git', 'merge', '--abort', check=False)
            sys.exit('merge conflict, aborted: rebase the branch on main first\n' + merged.stdout)
        sh('python3', 'tools/progress_index.py')
        if sh('git', 'status', '--porcelain').stdout.strip():
            sh('git', 'add', '-A', 'docs')
            sh('git', 'commit', '-qm', 'docs: regenerate shared index after ' + args.branch)
        print('merged', args.branch, 'at', sh('git', 'rev-parse', '--short', 'HEAD').stdout.strip())
    finally:
        sh('python3', 'tools/session.py', 'release', 'shared', '--owner', args.owner, check=False)


if __name__ == '__main__':
    main()
