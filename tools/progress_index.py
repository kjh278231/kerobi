#!/usr/bin/env python3
"""Regenerate shared doc blocks from docs/progress/<task>.md so sprint branches never edit them.

Entry file: header lines `date: ISO`, `area: M1|M2|M3`, then optional sections
  ## 11 / ## 12      paragraph put (newest first) between <!-- progress:begin/end --> in docs/11_progress.md / 12_roadmap_diagram.md
  ## 13 / ## 14      table rows; a row whose first cell equals an existing row's replaces it, else it is added after the area's last row
  ## handoff         newest 3 per area between <!-- handoff:begin/end --> in docs/handoff/<area>.md
  python3 tools/progress_index.py [--check] [--self-test] [--init]   # --init: add markers once to the existing docs
"""
import re, sys, tempfile
from pathlib import Path

BLOCK = '<!-- {0}:begin -->\n{1}\n<!-- {0}:end -->'


def load(folder):
    out = []
    for path in sorted(folder.glob('*.md')):
        if path.name == 'README.md':
            continue
        head, sections, cur = {}, {}, None
        for line in path.read_text().splitlines():
            if line.startswith('## '):
                cur = sections.setdefault(line[3:].strip(), [])
            elif cur is not None:
                cur.append(line)
            elif ':' in line:
                key, val = line.split(':', 1)
                head[key.strip()] = val.strip()
        if 'date' not in head or 'area' not in head:
            raise ValueError(f'{path.name}: date:/area: header required')
        out.append(dict(id=path.stem, head=head, sec={k: '\n'.join(v).strip() for k, v in sections.items()}))
    return sorted(out, key=lambda e: e['head']['date'], reverse=True)


def put_block(text, tag, body):
    pat = re.compile(r'<!-- %s:begin -->.*?<!-- %s:end -->' % (tag, tag), re.S)
    if not pat.search(text):
        raise ValueError(f'marker {tag} missing')
    return pat.sub(lambda _: BLOCK.format(tag, body), text)


def upsert_rows(text, rows):
    lines = text.split('\n')
    for row in rows:
        key = row.split('|')[1].strip()
        idx = next((i for i, l in enumerate(lines) if l.startswith('| ' + key + ' |')), None)
        if idx is not None:
            lines[idx] = row
            continue
        prefix = key[:2] + '-'
        last = max((i for i, l in enumerate(lines) if l.startswith('| ' + prefix)), default=None)
        if last is None:
            raise ValueError(f'no table rows for area of {key}')
        lines.insert(last + 1, row)
    return '\n'.join(lines)


def build(root):
    entries = load(root / 'docs/progress')
    plan = {}
    for tag, name in [('progress', 'docs/11_progress.md'), ('progress', 'docs/12_roadmap_diagram.md')]:
        sec = '11' if name.endswith('11_progress.md') else '12'
        body = '\n\n'.join(e['sec'][sec] for e in entries if e['sec'].get(sec))
        plan[name] = put_block((root / name).read_text(), tag, body)
    for sec, name in [('13', 'docs/13_work_units.md'), ('14', 'docs/14_sprint_plan.md')]:
        rows = [r for e in reversed(entries) for r in e['sec'].get(sec, '').split('\n') if r.startswith('|')]
        plan[name] = upsert_rows((root / name).read_text(), rows)
    for area in ('M1', 'M2', 'M3'):
        name = f'docs/handoff/{area}.md'
        body = '\n\n'.join(f"### {e['id']} ({e['head']['date']})\n\n{e['sec']['handoff']}"
                           for e in [e for e in entries if e['head']['area'] == area and e['sec'].get('handoff')][:3])
        plan[name] = put_block((root / name).read_text(), 'handoff', body)
    return plan


def run(root, check):
    changed = []
    for name, new in build(root).items():
        path = root / name
        if path.read_text() != new:
            changed.append(name)
            if not check:
                path.write_text(new)
    print(('stale: ' if check else 'updated: ') + ', '.join(changed) if changed else 'up to date')
    return int(bool(changed) and check)


def init(root):
    """Insert empty markers after the title of the generated docs (legacy text stays below them)."""
    for name, tag in [('docs/11_progress.md', 'progress'), ('docs/12_roadmap_diagram.md', 'progress')] + \
                     [(f'docs/handoff/{a}.md', 'handoff') for a in ('M1', 'M2', 'M3')]:
        path = root / name
        text = path.read_text()
        if f'<!-- {tag}:begin -->' not in text:
            head, _, rest = text.partition('\n\n')
            path.write_text(head + '\n\n' + BLOCK.format(tag, '') + '\n\n' + rest)
            print('markers added:', name)


def self_test():
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        for name in ['docs/progress', 'docs/handoff']:
            (root / name).mkdir(parents=True)
        mark = lambda t: f'# t\n\n<!-- {t}:begin -->\n<!-- {t}:end -->\n\nlegacy\n'
        (root / 'docs/11_progress.md').write_text(mark('progress'))
        (root / 'docs/12_roadmap_diagram.md').write_text(mark('progress'))
        for a in ('M1', 'M2', 'M3'):
            (root / f'docs/handoff/{a}.md').write_text(mark('handoff'))
        (root / 'docs/13_work_units.md').write_text('| M2-01 | a |\n| M2-02 | b |\n')
        (root / 'docs/14_sprint_plan.md').write_text('| M2-01 | x |\n| M2-02 | y |\n| M1-01 | z |\n')
        (root / 'docs/progress/M2-01.md').write_text('date: 2026-11-11T01:00\narea: M2\n## 11\nold\n## 14\n| M2-01 | done |\n')
        (root / 'docs/progress/M2-03.md').write_text('date: 2026-11-11T02:00\narea: M2\n## 11\nnew\n## 14\n| M2-03 | new |\n## handoff\nnext\n')
        assert run(root, True) == 1
        assert run(root, False) == 0 and run(root, True) == 0
        p = (root / 'docs/11_progress.md').read_text()
        assert p.index('new') < p.index('old') < p.index('legacy')
        assert (root / 'docs/14_sprint_plan.md').read_text() == '| M2-01 | done |\n| M2-02 | y |\n| M2-03 | new |\n| M1-01 | z |\n'
        assert 'next' in (root / 'docs/handoff/M2.md').read_text()
        (root / 'docs/progress/bad.md').write_text('nothing\n')
        try:
            build(root)
            raise SystemExit('bad header accepted')
        except ValueError:
            pass
    print('self-test ok')


if __name__ == '__main__':
    if '--self-test' in sys.argv:
        self_test()
    elif '--init' in sys.argv:
        init(Path(__file__).resolve().parents[1])
    else:
        sys.exit(run(Path(__file__).resolve().parents[1], '--check' in sys.argv))
