"""M3-04c-20: 입력 전용 Rust 재생을 원본 round 후상태와 직접 비교한다."""
import argparse
from collections import Counter
import copy
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

import push_round_check as oracle
from push_recompute_rust import compare

ROOT = Path(__file__).resolve().parents[1]
D = ROOT / 'bench/oracle/m3_push_round_rust'
TRACE = ROOT / 'target/m3_push_round_rust/trace.txt'
SOURCE = Path('crates/lp/src/crossover_round.rs')
SCHEMA = 'M3-04c-20-v1'
STATE = ('m', 'total', 'candidates', 'position', 'deferred', 'failures', 'marks', 'support',
         'approved', 'count_sum', 'successes', 'request', 'status', 'numeric', 'failure', 'retry')


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def canonical(st, exit='continue', calls=None, code=0):
    return dict({k: copy.deepcopy(st[k]) for k in STATE if k in st}, exit=exit, return_code=code, calls=calls or [])


def cases():
    data, manifest = oracle.records(), oracle.load(oracle.D/'manifest.json')
    oracle.check_files(manifest)
    oracle.controls(data, manifest)
    oracle.acceptance(data)
    out = []
    for model in ('arms', 'retry', 'interior'):
        pending = {}
        approved = 0
        first = True
        def add(kind, before, after, exit='continue', calls=None, **extra):
            # These are independent state boundaries; whole-loop linkage is task22.
            x = copy.deepcopy(before)
            x.update(extra)
            out.append(dict(name=f'{model}/{len(out)}/{kind}', kind=kind, origin='natural',
                            input=x, result=canonical(after, exit, calls)))
        for e in data[model][0]['events']:
            tag = e['event']
            st = e.get('state')
            if tag == 'roundctl:pass:enter':
                pending['pass'] = st
                approved, first = 0, True
            elif tag == 'roundctl:pass:ready':
                add('pass', pending.pop('pass'), st)
            elif tag == 'roundctl:accepted':
                before = copy.deepcopy(st)
                before['approved'] = None if first else approved
                add('begin_accept' if first else 'accept', before, st)
                approved += 1
                first = False
            elif tag == 'roundctl:defer:enter':
                pending['defer'] = e
                first = False
            elif tag == 'roundctl:defer:leave':
                entry = pending.pop('defer')
                add('defer', entry['state'], st, enter=entry['selected']['enter'])
            elif tag == 'roundctl:btran:prepare':
                pending['btran'] = e
            elif tag == 'roundctl:btran:enter':
                pending['unit'] = e['input']
            elif tag == 'roundctl:btran:leave':
                pending['raw'] = e['raw']
            elif tag == 'roundctl:btran:applied':
                entry = pending.pop('btran')
                add('btran', entry['state'], st, calls=[dict(event='btran', input=pending.pop('unit'))],
                    leave=entry['selected']['leave'], raw=pending.pop('raw'))
            elif tag == 'roundctl:exhausted':
                pending['exhausted'] = st
            elif tag == 'roundctl:restart':
                add('gate', pending['exhausted'], st)
                pending['restart'] = st
            elif tag == 'roundctl:support':
                add('support', pending['restart'], st)
            elif tag == 'roundctl:factor':
                add('exhausted', pending.pop('exhausted'), st, exit='factor')
                pending.pop('restart')
                pending['factor'] = st
                pending['calls'] = []
            elif tag == 'factor:enter' and 'factor' in pending:
                pending['factor_request'] = e['request']
            elif tag == 'factor:leave' and 'factor' in pending:
                reply = dict(ret=e['ret'], request=e['request'])
                reply.update({k: e['factor_ctl']['state'][k] for k in ('status', 'numeric', 'failure')})
                pending['calls'].append(dict(event='factor', request=pending.pop('factor_request'), reply=reply))
            elif tag == 'roundctl:factor:return':
                calls = pending.pop('calls')
                add('wrapper', pending.pop('factor'), st, exit='recompute', calls=calls,
                    replies=[c['reply'] for c in calls])
            elif tag == 'roundctl:recomputed':
                pending['clear'] = st
            elif tag == 'roundctl:clear':
                add('clear', pending.pop('clear'), st)
            elif tag == 'roundctl:end':
                if 'exhausted' in pending:
                    add('exhausted', pending.pop('exhausted'), st, exit='end')
                elif 'clear' in pending:
                    add('clear', pending.pop('clear'), st, exit='continue')
                else:
                    raise oracle.Fail(model+'.unpaired end')
        oracle.equal({}, pending, model+'.pending')
    for e in data['synthetic'][0]['events']:
        x, r = copy.deepcopy(e['input']), e['result']
        x.update(total=len(x['candidates']), retry=0)
        exit = {101:'continue', 102:'factor', 103:'end', 104:'continue', 105:'recompute',
                106:'retry' if r['retry'] else 'error'}[r['exit']]
        out.append(dict(name='synthetic/'+e['name'], kind=e['kind'], origin='synthetic', input=x,
                        result=canonical(r, exit, r['calls'], r['return_code'])))
    oracle.equal(len(out), len({c['name'] for c in out}), 'unique cases')
    oracle.equal(216, len(out), '110 natural boundaries + 106 synthetic cases')
    oracle.equal(106, sum(c['origin']=='synthetic' for c in out), 'all synthetic cases')
    return out


def project(cs):
    lines, expected = [], {}
    def arr(a): return [len(a), *a]
    for c in cs:
        x = c['input']
        oracle.equal([0,0,0], x.get('batch', [0,0,0]), c['name']+'.batch0')
        fields = [c['name'], c['kind'], int('approved' in c['result']), x['m'], *arr(x['candidates']), x['position'], x['deferred'],
                  *arr(x['failures']), int(x['marks'] is not None)]
        if x['marks'] is not None: fields += arr(x['marks'])
        fields += [x['support'], x.get('approved', '_'), x['count_sum'], x['successes'], x['request'],
                   x['status'], x['numeric'], x['failure'], x['retry'], x['attempt'], x['numeric_limit'],
                   x.get('enter',0), x.get('leave',0), int('raw' in x)]
        if 'raw' in x:
            raw = x['raw']
            fields += [raw['count'], *arr(raw['indices'] or []), *arr(raw['values'])]
        replies = x.get('replies', [])
        fields.append(len(replies))
        for r in replies: fields += [r['ret'], *(r.get(k,'_') for k in ('request','status','numeric','failure'))]
        lines.append(' '.join('_' if f is None else str(f) for f in fields))
        expected[c['name']] = c['result']
    body = '\n'.join(lines)+'\n'
    return SCHEMA+' '+hashlib.sha256(body.encode()).hexdigest()+'\n'+body, expected


def self_test(txt, expected, header):
    results = []
    def reject(name, lines):
        try: compare('\n'.join(lines)+'\n',expected,header)
        except (oracle.Fail,KeyError,TypeError,ValueError) as exc:
            results.append(dict(name=name,first_difference=str(exc))); return
        raise oracle.Fail('NOT REJECTED '+name)
    lines = txt.splitlines()
    reject('zero-target',lines[:1]); reject('missing-case',lines[:-1])
    reject('input-hash-zero',[SCHEMA+' '+'0'*64]+lines[1:])
    reject('case-order',lines[:1]+[lines[2],lines[1]]+lines[3:])
    # Every output field for every case must be observed, including preserved state.
    for i,(name,want) in enumerate(expected.items(),1):
        for k,v in want.items():
            changed=copy.deepcopy(want)
            changed[k]=v+1 if type(v) is int else None if v is not None else 0
            altered=lines.copy();altered[i]=name+';'+json.dumps(changed)
            reject(name+'/'+k,altered)
    return results


MUTATIONS = [
    ('begin-nonzero', 'state.approved = Some(0);', 'state.approved = Some(1);'),
    ('accepted-double', 'initialize approved") += 1;', 'initialize approved") += 2;'),
    ('defer-wrong-column', 'state.candidates[state.deferred] = enter;', 'state.candidates[state.deferred] = 0;'),
    ('defer-no-increment', 'state.failures[enter as usize] += 1;', 'state.failures[enter as usize] += 0;'),
    ('defer-double-increment', 'state.failures[enter as usize] += 1;', 'state.failures[enter as usize] += 2;'),
    ('defer-no-prefix-advance', 'state.deferred += 1;', 'state.deferred += 0;'),
    ('compact-no-copy', 'state.candidates.copy_within(state.position..total, state.deferred);', 'let _ = total;'),
    ('compact-keep-length', 'state.candidates.truncate(retained);', 'let _ = retained;'),
    ('compact-reverse', 'state.candidates.truncate(retained);', 'state.candidates.truncate(retained); state.candidates.reverse();'),
    ('compact-deduplicate', 'state.candidates.truncate(retained);', 'state.candidates.truncate(retained); state.candidates.sort(); state.candidates.dedup();'),
    ('compact-failure-reset', 'state.position = 0;', 'state.position = 0; state.failures.fill(0);'),
    ('compact-stat-reset', 'state.position = 0;', 'state.position = 0; state.count_sum = 0.0; state.successes = 0;'),
    ('pass-no-mark-clear', 'if state.support != 0 { clear_marks(state); }', 'if false { clear_marks(state); }'),
    ('pass-clear-support0', 'if state.support != 0 { clear_marks(state); }', 'clear_marks(state);'),
    ('all-deferred-ends-supported', 'if state.deferred == 0 || state.support == 0', 'if state.deferred == 0 || true'),
    ('empty-does-not-end', 'if state.deferred == 0 || state.support == 0', 'if state.support == 0'),
    ('all-deferred-keep-support', 'state.support = 0;\n    }\n    RoundExit::Continue', 'state.support = 1;\n    }\n    RoundExit::Continue'),
    ('sqrt-equality-release', '.sqrt() > f64::from(approved)', '.sqrt() >= f64::from(approved)'),
    ('sqrt-omitted', '(state.m as f64).sqrt()', 'state.m as f64'),
    ('support-no-clear', '} else if state.support != 0 {\n        clear_marks(state);', '} else if state.support != 0 {\n        let _ = state;'),
    ('restart-request6', 'state.request = 10;', 'state.request = 6;'),
    ('btran-skip', 'if state.support == 0 { return RoundExit::Continue; }', 'if true { return RoundExit::Continue; }'),
    ('btran-wrong-unit', 'values: vec![1.0]', 'values: vec![-1.0]'),
    ('btran-overwrite-union', 'for &row in &raw.indices { marks[row as usize] = 1; }', 'marks.fill(0); for &row in &raw.indices { marks[row as usize] = 1; }'),
    ('btran-ignore-zero-value', 'for &row in &raw.indices { marks[row as usize] = 1; }', 'for (&row,&v) in raw.indices.iter().zip(&raw.values) { if v != 0.0 { marks[row as usize] = 1; } }'),
    ('btran-no-mark', 'for &row in &raw.indices { marks[row as usize] = 1; }', 'for &row in &raw.indices { marks[row as usize] = 0; }'),
    ('dense-support0', 'state.support = -1;', 'state.support = 0;'),
    ('dense-no-clear', 'state.support = -1;\n        clear_marks(state);', 'state.support = -1;'),
    ('dense-empty-factor', 'if state.candidates.is_empty() { return RoundExit::End; }', 'if false { return RoundExit::End; }'),
    ('factor-no-repeat', 'if ret != 1 { break ret; }', 'if ret >= 0 { break ret; }'),
    ('return12-as-error', 'if ret == 12', 'if ret == 13'),
    ('status-boundary', 'state.status < -999', 'state.status <= -999'),
    ('status-restored0', 'state.status = 1;', 'state.status = 0;'),
    ('ignore-attempt', 'frame.attempt == 0', 'frame.attempt <= 1'),
    ('ignore-failure', 'state.failure != 0 || state.numeric', 'false || state.numeric'),
    ('numeric-strict', 'state.numeric >= frame.numeric_limit', 'state.numeric > frame.numeric_limit'),
    ('clear-request-early', '    RoundExit::Recompute\n', '    state.request = 0; RoundExit::Recompute\n'),
    ('clear-no-request', '    state.request = 0;\n}', '    state.request = 10;\n}'),
    ('clear-approved-lost', '    state.request = 0;\n}', '    state.request = 0; state.approved = None;\n}'),
]


def mutate_rust(path, expected, header):
    scratch=ROOT/'target/m3_round_mutations'
    if scratch.exists(): shutil.rmtree(scratch)
    scratch.mkdir(parents=True)
    for file in ('Cargo.toml','Cargo.lock'):
        shutil.copy2(ROOT/file,scratch/file)
    shutil.copytree(ROOT/'crates',scratch/'crates')
    (scratch/'bench/oracle/m3_push_round_rust').mkdir(parents=True)
    shutil.copy2(D/'input.txt',scratch/'bench/oracle/m3_push_round_rust/input.txt')
    original=(ROOT/SOURCE).read_text(); results=[]
    trace=scratch/'target/m3_push_round_rust/trace.txt'
    for name,old,new in MUTATIONS:
        oracle.equal(1,original.count(old),'unique mutation '+name)
        (scratch/SOURCE).write_text(original.replace(old,new,1)); trace.unlink(missing_ok=True)
        proc=subprocess.run(['cargo','test','-p','rsolver-lp','--test','push_round'],cwd=scratch,
                            text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
        if proc.returncode or not trace.exists():
            results.append(dict(name=name,result='build-or-test-failed',log=proc.stdout)); continue
        try: compare(trace.read_text(),expected,header)
        except (oracle.Fail,KeyError,TypeError,ValueError) as exc:
            results.append(dict(name=name,result='rejected',first_difference=str(exc)))
        else: results.append(dict(name=name,result='survived'))
    path.write_text(json.dumps(results,ensure_ascii=False,indent=2)+'\n')
    oracle.require(results,'source mutation targets required')
    for r in results: oracle.equal('rejected',r['result'],'behavioral mutant '+r['name'])
    return dict(rejected=len(results))


def manifest():
    files=[SOURCE,Path('crates/lp/src/lib.rs'),Path('crates/lp/tests/push_round.rs'),
           Path('tools/push_round_rust.py'),Path('tools/push_recompute_rust.py'),
           Path('bench/oracle/m3_push_round/manifest.json')]
    files += [p.relative_to(ROOT) for p in sorted(D.rglob('*')) if p.is_file() and p.name!='manifest.json']
    return dict(task='M3-04c-20',baseline='55e09d6',binary_sha256=oracle.BINARY_SHA,
                oracle_manifest_sha256=sha(ROOT/oracle.D/'manifest.json'),files={str(p):sha(ROOT/p) for p in files})


def main():
    p=argparse.ArgumentParser(description=__doc__)
    g=p.add_mutually_exclusive_group(required=True)
    for flag in ('write','compare','self-test','write-manifest'):g.add_argument('--'+flag,action='store_true')
    p.add_argument('--mutate-rust',type=Path)
    a=p.parse_args(); cs=cases(); source,expected=project(cs); header=source.splitlines()[0]
    D.mkdir(parents=True,exist_ok=True)
    if a.write:
        (D/'input.txt').write_text(source);result=dict(written=len(cs))
    else:
        oracle.equal(source,(D/'input.txt').read_text(),'fixed input-only projection')
        txt=TRACE.read_text();result=compare(txt,expected,header)
        result.update(origins=dict(Counter(c['origin'] for c in cs)),kinds=dict(Counter(c['kind'] for c in cs)))
        if a.mutate_rust: result['source_mutations']=mutate_rust(a.mutate_rust,expected,header)
        elif a.self_test:
            rejected=self_test(txt,expected,header)
            (D/'self-test.json').write_text(json.dumps(rejected,ensure_ascii=False,indent=2)+'\n')
            result['rejected']=len(rejected)
        else:(D/'comparison.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
        if a.write_manifest:
            (D/'trace.txt').write_text(txt)
            (D/'manifest.json').write_text(json.dumps(manifest(),ensure_ascii=False,indent=2)+'\n')
    print(json.dumps(result,ensure_ascii=False))


if __name__=='__main__':
    try:main()
    except (oracle.Fail,KeyError,TypeError,ValueError,IndexError) as exc:sys.exit('FAIL '+str(exc))
