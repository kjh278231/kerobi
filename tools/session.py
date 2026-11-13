#!/usr/bin/env python3
"""Portable session status, local claims and offline handoff bundles (stdlib only)."""
import argparse
import datetime
import fcntl
import json
import os
from pathlib import Path
import re
import socket
import subprocess
import sys
import tarfile
import tempfile
import uuid

ROOT = Path(__file__).resolve().parents[1]


def git(*args, env=None):
    return subprocess.check_output(['git', '-C', str(ROOT), *args], env=env, text=True).strip()


def common_dir():
    return Path(git('rev-parse', '--path-format=absolute', '--git-common-dir'))


def claim(args):
    if not re.fullmatch(r'M[123]-\d{2}[a-z](?:-\d+[a-z]?)*|shared', args.task):
        raise ValueError('하위 작업 번호 또는 shared를 지정하세요.')
    lane = args.task[:5] if args.task != 'shared' else 'shared'  # M2-06 group: two groups of one area may run together
    folder = common_dir() / 'rsolver-sessions'
    folder.mkdir(exist_ok=True)
    with (folder / 'lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        path = folder / (lane + '.json')
        if args.command == 'release':
            current = json.loads(path.read_text())
            if current['owner'] != args.owner or current['task'] != args.task:
                raise ValueError('작업 번호와 owner가 기존 점유와 다릅니다. 자동 해제하지 않습니다.')
            path.unlink()
            print('해제:', args.task)
            return
        if path.exists():
            raise ValueError('이미 점유됨: ' + path.read_text())
        record = dict(task=args.task, owner=args.owner, host=socket.gethostname(),
                      branch=git('branch', '--show-current'), worktree=str(ROOT),
                      started=datetime.datetime.now(datetime.timezone.utc).isoformat())
        path.write_text(json.dumps(record, ensure_ascii=False, indent=2) + '\n')
        print(json.dumps(record, ensure_ascii=False, indent=2))


def status():
    print(git('status', '--short', '--branch'))
    print(git('worktree', 'list'))
    for path in sorted((common_dir() / 'rsolver-sessions').glob('M*.json')):
        print(path.read_text().strip())
    shared = common_dir() / 'rsolver-sessions/shared.json'
    if shared.exists():
        print(shared.read_text().strip())
    print('인수인계: docs/handoff/README.md → M1.md / M2.md / M3.md')
    print('점유는 이 git common-dir에서만 유효합니다. 다른 clone·컴퓨터와 공유되지 않습니다.')


def doctor():
    required = ['AGENTS.md', 'docs/14_sprint_plan.md', 'docs/handoff/README.md']
    missing = False
    for name in required:
        ok = (ROOT / name).exists()
        print(('OK ' if ok else 'MISSING ') + name)
        missing |= not ok
    for name in ['decomp/c', 'decomp/asm', 'decomp/index.tsv', 'decomp/callgraph.tsv', 'notes', 'bench/data']:
        path = ROOT.parent / 're' / name
        print(('OK ' if path.exists() else 'MISSING ') + '../re/' + name)
        missing |= not path.exists()
    for binary in ['git', 'cargo', 'rustc', 'python3']:
        try:
            result = subprocess.run([binary, '--version'], capture_output=True, text=True, check=True)
            print(result.stdout.strip().splitlines()[0])
        except (OSError, subprocess.CalledProcessError):
            print('MISSING ' + binary)
            missing = True
    print('Python 확장·가상환경·Gurobi 13.0.3과 라이선스는 대상 컴퓨터에서 별도 확인합니다.')
    return int(missing)


def export(dest, include_reference=False):
    dest = Path(dest).resolve()
    if dest == ROOT or ROOT in dest.parents:
        raise ValueError('출력은 저장소 바깥의 새 디렉터리여야 합니다.')
    dest.mkdir(parents=True, exist_ok=False)
    before = git('status', '--porcelain=v1')
    ref = 'refs/heads/handoff-' + uuid.uuid4().hex
    with tempfile.TemporaryDirectory() as temp:
        env = os.environ.copy()
        env['GIT_INDEX_FILE'] = str(Path(temp) / 'index')
        git('read-tree', 'HEAD', env=env)
        git('add', '-A', env=env)
        tree = git('write-tree', env=env)
        env.update(GIT_AUTHOR_NAME='rsolver handoff', GIT_AUTHOR_EMAIL='handoff@localhost',
                   GIT_COMMITTER_NAME='rsolver handoff', GIT_COMMITTER_EMAIL='handoff@localhost')
        commit = git('commit-tree', tree, '-p', 'HEAD', '-m', 'Offline session handoff snapshot', env=env)
        try:
            git('update-ref', ref, commit)
            git('bundle', 'create', str(dest / 'rsolver.bundle'), '--all')
        finally:
            git('update-ref', '-d', ref)
    manifest = dict(snapshot=commit, branch=ref.removeprefix('refs/heads/'),
                    source_head=git('rev-parse', 'HEAD'), source_status=before,
                    note='tracked + non-ignored untracked files; ignored files and ../re excluded')
    if include_reference:
        reference = ROOT.parent / 're'
        names = ['decomp', 'notes', 'scripts', 'bench', 'README.md']
        for name in names:
            if not (reference / name).exists():
                raise ValueError('외부 자료 누락: ' + str(reference / name))
        with tarfile.open(dest / 'reference.tar.gz', 'w:gz') as archive:
            for name in names:
                archive.add(reference / name, arcname='re/' + name)
        manifest['reference'] = 'reference.tar.gz'
        manifest['note'] = 'tracked + non-ignored untracked files; reference included separately; ignored files excluded'
    assert before == git('status', '--porcelain=v1'), '내보내기 도중 원본 worktree가 변경됐습니다. 새 패키지를 만드세요.'
    git('bundle', 'verify', str(dest / 'rsolver.bundle'))
    (dest / 'manifest.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
    print('패키지:', dest)
    print('복원 branch:', manifest['branch'])
    print('../re와 환경 준비는 docs/handoff/README.md를 따르세요. 점유 정보는 전달하지 않습니다.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    commands.add_parser('status')
    commands.add_parser('doctor')
    for cmd in ['claim', 'release']:
        p = commands.add_parser(cmd)
        p.add_argument('task')
        p.add_argument('--owner', required=True, help='session identifier, e.g. laptop-a-session-1')
    p = commands.add_parser('export')
    p.add_argument('destination')
    p.add_argument('--include-reference', action='store_true', help='include ../re source maps and benchmark inputs')
    args = parser.parse_args()
    try:
        if args.command in ('claim', 'release'):
            claim(args)
        elif args.command == 'status':
            status()
        elif args.command == 'doctor':
            return doctor()
        else:
            export(args.destination, args.include_reference)
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        print(str(error), file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
