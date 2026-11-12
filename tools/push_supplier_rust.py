"""M3-04c-14: before와 외부 FTRAN 결과만 Rust에 주고 원본 supplier 반환/후상태를 비교한다.

--write: 고정69입력 작성; --compare: Rust trace 비교; --self-test: 비교기 변조 거부;
--mutate-rust PATH: Rust 동작 변이마다 재생/비교, 원본 소스 복구, 새 결과 파일 기록.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys

import push_supplier_check as oracle
from push_supplier_synthetic import bits

ROOT = Path(__file__).resolve().parents[1]
DIRECTORY = ROOT / 'bench/oracle/m3_push_supplier_rust'
INPUT = DIRECTORY / 'input.txt'
TRACE = ROOT / 'target/m3_push_supplier_rust/trace.txt'
SOURCE = ROOT / 'crates/lp/src/crossover_supplier.rs'
HEADER = 'M3-04c-14-v1'
CANARY = dict(count=1, indices=[0], values=[bits(-0.)])


def vector(v):
    return ' '.join(map(str, [v['count'], *(v['indices'] or []), *v['values']]))


def cases():
    data, manifest = oracle.records(), oracle.load(f'{oracle.D}/manifest.json')
    oracle.check_files(manifest)
    oracle.controls(data, manifest)
    oracle.acceptance(data)
    out = []
    for name in ('arms', 'retry', 'interior'):
        for k, u in enumerate(oracle.units(data[name][0])):
            out.append((f'{name}/{k}', u))
    out.extend((f'synthetic/{u["name"]}', u) for u in data['synthetic'][0]['events'])
    oracle.equal(69, len(out), '대표 입력 개수')
    return out


def project(cs):
    lines = [HEADER]
    expected = {}
    for name, u in cs:
        b, a, m, n, model = u['before'], u['after'], u['m'], u['n'], u['model']
        starts = [int.from_bytes(bytes.fromhex(x), 'little', signed=True) if isinstance(x, str) else x
                  for x in model['starts']]
        marks = b['support_marks']
        f = u['ftran'][0] if u['ftran'] else None
        spike_before = (f.get('spike_before') if f else None) or CANARY
        spike_after = (f.get('spike_after') if f else None) or spike_before
        fields = [name, m, n, len(model['rows']), u['drop'], b['total'], b['position'], b['deferred'],
                  b['support'], b['count_sum'], b['successes'], *starts, *model['lengths'],
                  *model['rows'], *model['values'], *b['candidates'], *b['failures'],
                  -1 if marks is None else len(marks), *(marks or []), vector(spike_before),
                  int(f is not None)]
        if f:
            fields += [vector(f['raw']), vector(spike_after)]
        lines.append(' '.join(map(str, fields)))
        # Expected values are the original capture, not oracle.replay's computed state.
        want = {k: str(a[k]) for k in ('position', 'deferred', 'support', 'successes', 'count_sum')}
        for k in ('candidates', 'failures'):
            want[k] = ' '.join(map(str, a[k]))
        want['support_marks'] = 'null' if a['support_marks'] is None else ' '.join(map(str, a['support_marks']))
        want.update(ret=str(u['ret']), ftran_count=str(len(u['ftran'])),
                    alpha=vector(u['alpha'] if f else CANARY),
                    input=vector(f['input'] if f else CANARY),
                    spike=vector(spike_after if f and b['support'] <= 0 else spike_before),
                    work=bits(42.75 if f else 42.25), output_same='true', spike_same='true',
                    backend_input=vector(f['input']) if f else 'null')
        if f:
            want['spike_present'] = str(f['aliases']['spike_f1c8']).lower()
            want['spike_before'] = vector(spike_before) if f['aliases']['spike_f1c8'] else 'null'
        else:
            want['spike_present'] = 'false'
            want['spike_before'] = 'null'
        expected[name] = want
    return '\n'.join(lines) + '\n', expected


def compare(txt, expected):
    lines = txt.splitlines()
    oracle.require(lines and lines[0] == HEADER, 'trace schema')
    oracle.equal(len(expected), len(lines)-1, 'trace 사례 수')
    fields = 0
    for line, (name, want) in zip(lines[1:], expected.items()):
        parts = line.split(';')
        oracle.equal(name, parts[0], '사례 순서')
        got = {}
        for item in parts[1:]:
            key, value = item.split(' ', 1)
            oracle.require(key not in got, name+'.duplicate '+key)
            got[key] = value
        oracle.equal(want, got, name)
        fields += len(want)
    return dict(cases=len(expected), fields=fields, natural=51, synthetic=18,
                success=56, exhausted=13, rule='ordered integers / f64 LE hex exact incl signed zero')


def self_test(txt, expected):
    results = []
    def bad(name, edit):
        lines = txt.splitlines()
        edit(lines)
        try:
            compare('\n'.join(lines)+'\n', expected)
        except (oracle.Fail, ValueError) as exc:
            results.append(dict(name=name, first_difference=str(exc)))
            return
        raise oracle.Fail('변조 미검출: '+name)
    def replace_field(lines, key, value, case_index=1):
        parts = lines[case_index].split(';')
        targets = [i for i, p in enumerate(parts) if p.startswith(key+' ')]
        oracle.require(len(targets)==1, '변조 대상 '+key)
        parts[targets[0]] = key+' '+value
        lines[case_index] = ';'.join(parts)
    bad('empty-targets', lambda ls: ls.__setitem__(slice(1, None), []))
    bad('missing-case', lambda ls: ls.pop())
    bad('case-order', lambda ls: ls.__setitem__(slice(1, 3), ls[1:3][::-1]))
    for key in next(iter(expected.values())):
        bad('field-'+key, lambda ls, k=key: replace_field(ls, k, 'corrupted'))
    bad('duplicate-field', lambda ls: ls.__setitem__(1, ls[1]+';ret 0'))
    bad('missing-field', lambda ls: ls.__setitem__(1, ';'.join(ls[1].split(';')[:-1])))
    for name, key in [('synthetic/structural-sparse', 'alpha'), ('synthetic/dense-boundaries', 'alpha'),
                      ('synthetic/prefix-preserved', 'candidates'), ('synthetic/dense-negative-count', 'count_sum'),
                      ('synthetic/all-failure', 'ftran_count')]:
        index = list(expected).index(name)+1
        bad(name+'/'+key, lambda ls, k=key, i=index: replace_field(ls, k, 'corrupted', i))
    return results


# Each variant has a fixed expectation: rejection by an actual output difference on the fixed69inputs.
MUTATIONS = [
    ('failure-two', 'state.failures[j] >= 3', 'state.failures[j] >= 2'),
    ('failure-four', 'state.failures[j] >= 3', 'state.failures[j] >= 4'),
    ('ignore-support', '!failed && state.support > 0 && {', '!failed && state.support > 100 && {'),
    ('positive-marks-only', 'marks[row as usize] != 0', 'marks[row as usize] > 1'),
    ('ignore-slack-conflict', 'marks[j - model.n] != 0', 'false'),
    ('no-normalization', 'state.support = state.support.max(0);', 'state.support = state.support;'),
    ('defer-wrong-candidate', 'state.candidates[state.deferred] = j as i32;', 'state.candidates[state.deferred] = 0;'),
    ('increment-failures', 'state.deferred += 1;', 'state.deferred += 1; state.failures[j] += 1;'),
    ('reverse-column', 'input.values.extend_from_slice(&model.values[start..end]);',
     'input.values.extend_from_slice(&model.values[start..end]); input.indices.reverse(); input.values.reverse();'),
    ('negative-slack', 'input.values.push(1.0);', 'input.values.push(-1.0);'),
    ('never-spike', 'if state.support == 0 { Some(&mut buffers.spike) } else { None }',
     'if state.support < 0 { Some(&mut buffers.spike) } else { None }'),
    ('always-spike', 'if state.support == 0 { Some(&mut buffers.spike) } else { None }',
     'Some(&mut buffers.spike)'),
    ('no-success-counter', 'state.successes += 1;', 'state.successes += 0;'),
    ('absolute-count', 'f64::from(raw_count) + state.count_sum', 'f64::from(raw_count.abs()) + state.count_sum'),
    ('compressed-statistics', 'return j as i32;',
     'state.count_sum += f64::from(alpha.count - raw_count); return j as i32;'),
    ('keep-equal-drop', 'model.drop < value.abs()', 'model.drop <= value.abs()'),
    ('drop-negative-values', 'model.drop < value.abs()', 'model.drop < value'),
    ('lose-last-dense-row', 'for row in 0..model.m {', 'for row in 0..model.m.saturating_sub(1) {'),
    ('reverse-dense', 'alpha.values.truncate(kept);',
     'alpha.values.truncate(kept); alpha.indices.reverse(); alpha.values.reverse();'),
    ('sort-sparse', 'let raw_count = alpha.count;',
     'let raw_count = alpha.count; if raw_count >= 0 { alpha.indices.sort(); }'),
    ('wrong-return', 'return j as i32;', 'return j as i32 + 1;'),
    ('wrong-exhaustion', '    -1\n}', '    0\n}'),
]


def mutate(cs, expected):
    source = SOURCE.read_text()
    results = []
    try:
        for name, old, new in MUTATIONS:
            oracle.require(source.count(old)==1, 'Rust 변이 대상 유일성: '+name)
            SOURCE.write_text(source.replace(old, new))
            run = subprocess.run(['cargo', 'test', '-p', 'rsolver-lp', '--test', 'push_supplier'], cwd=ROOT,
                                 text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
            oracle.require(run.returncode==0, name+': 재생 자체 실패\n'+run.stdout[-2500:])
            try:
                compare(TRACE.read_text(), expected)
            except oracle.Fail as exc:
                results.append(dict(name=name, rejected=True, first_difference=str(exc)))
            else:
                raise oracle.Fail('Rust 변이 생존: '+name)
    finally:
        SOURCE.write_text(source)
        run = subprocess.run(['cargo', 'test', '-p', 'rsolver-lp', '--test', 'push_supplier'], cwd=ROOT,
                             text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        oracle.require(run.returncode==0, '원본 복구 후 재생 실패\n'+run.stdout[-2500:])
        compare(TRACE.read_text(), expected)
    return results


def main():
    p = argparse.ArgumentParser(description=__doc__)
    group = p.add_mutually_exclusive_group(required=True)
    for flag in ('write', 'compare', 'self-test'):
        group.add_argument('--'+flag, action='store_true')
    group.add_argument('--mutate-rust', type=Path)
    p.add_argument('--output', type=Path)
    a = p.parse_args()
    try:
        cs = cases()
        input_text, expected = project(cs)
        if a.write:
            DIRECTORY.mkdir(parents=True, exist_ok=True)
            INPUT.write_text(input_text)
            result = dict(input=str(INPUT.relative_to(ROOT)), sha256=hashlib.sha256(input_text.encode()).hexdigest(), cases=69)
        else:
            oracle.equal(input_text, INPUT.read_text(), '고정 입력 투영')
            if a.mutate_rust:
                oracle.require(not a.mutate_rust.exists(), '기존 결과 덮어쓰기 거부')
                result = dict(rejected=mutate(cs, expected))
                a.mutate_rust.write_text(json.dumps(result, ensure_ascii=False, indent=1)+'\n')
            else:
                txt = TRACE.read_text()
                result = compare(txt, expected)
                if a.self_test:
                    result['rejected'] = self_test(txt, expected)
        if a.output:
            oracle.require(not a.output.exists(), '기존 결과 덮어쓰기 거부')
            a.output.write_text(json.dumps(result, ensure_ascii=False, indent=1)+'\n')
        print(json.dumps(result, ensure_ascii=False, indent=1))
    except (oracle.Fail, OSError, ValueError, TypeError, KeyError, IndexError) as exc:
        print('불합격: '+str(exc), file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
