"""M3-04c-16: 고정 원본 입력/외부 backend 결과를 Rust에 주고 원본 후상태와 비교한다."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys

import push_factor_check as oracle

ROOT = Path(__file__).resolve().parents[1]
D = ROOT / 'bench/oracle/m3_push_factor_rust'
INPUT = D / 'input.txt'
TRACE = ROOT / 'target/m3_push_factor_rust/trace.txt'
SOURCE = ROOT / 'crates/lp/src/crossover_factor.rs'
HEADER = 'M3-04c-16-v1'
STATE_KEYS = ('request', 'numeric', 'failure', 'status', 'k')
FRAME_KEYS = ('period', 'attempt', 'numeric_limit', 'enter')
REPLY_KEYS = ('status', 'numeric', 'failure', 'request', 'k')


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def cases():
    data, manifest = oracle.records(), oracle.load(oracle.D / 'manifest.json')
    oracle.check_files(manifest)
    oracle.controls(data, manifest)
    oracle.acceptance(data)
    out = []
    for name in ('arms', 'retry', 'interior'):
        events = data[name][0]['events']
        for i, event in enumerate(events):
            tag = f'{name}/{event.get("call")}/{event.get("step")}'
            if event['event'] == 'factorctl:select':
                end = next(e for e in events[i+1:] if e['event'] in ('factorctl:call', 'factorctl:clear'))
                out.append(dict(name=tag+'/select', kind='select',
                                input=dict(event['state'], support=event['frame']['support']),
                                result=dict(request=end['state']['request'], call=end['event']=='factorctl:call', preserved=True)))
            if event['event'] == 'factorctl:call':
                restored = next(e for e in events[i+1:] if e['event']=='factorctl:restored')
                leaves = []
                for e in events[i+1:]:
                    if e['event']=='factorctl:restored':
                        break
                    if e['event']=='factor:leave':
                        leaves.append(e)
                oracle.require(leaves, tag+': backend missing')
                x = dict(event['state'], **event['frame'], enter=7)
                replies, calls = [], []
                request = x['request']
                for leave in leaves:
                    state = leave['factor_ctl']['state']
                    reply = dict(ret=leave['ret'], **{k: state[k] for k in STATE_KEYS})
                    replies.append(reply)
                    calls.append(dict(event='factor', request=request, alpha_current=True, spike_original=True, reply=reply))
                    request = state['request']
                state = restored['state']
                recompute = state['request'] != 0
                end = restored if recompute else next(e for e in events[i+1:] if e['event']=='factorctl:clear')
                result = dict(exit=201 if recompute else 200, **{k: state[k] for k in STATE_KEYS},
                              retry=0, period=end['frame']['period'], alpha_restored=restored['alpha_restored'],
                              spike_restored=restored['spike_restored'], contents_preserved=True, calls=calls)
                # Recompute body is excluded: request remains as observed at restored, before 0x1c3872.
                out.append(dict(name=tag+'/wrapper', kind='wrapper', input=x, replies=replies, qualities=[], result=result))
    out.extend(dict(copy.deepcopy(e), name='synthetic/'+e['name']) for e in data['synthetic'][0]['events'])
    oracle.equal(89, len(out), '28+6 natural / 55 synthetic inputs')
    return out


def project(cs):
    lines, expected = [HEADER], {}
    for case in cs:
        x = case['input']
        fields = [case['name'], case['kind']]
        if case['kind']=='select':
            fields += [x[k] for k in ('m','request','numeric','k','limit','q','C','E','L','U','support')]
        else:
            fields += [x[k] for k in (*STATE_KEYS, *FRAME_KEYS)]
            for key, first in (('replies','ret'), ('qualities','increment')):
                fields.append(len(case[key]))
                for reply in case[key]:
                    fields += [reply[first], *(reply.get(k, '_') for k in REPLY_KEYS)]
        lines.append(' '.join(map(str, fields)))
        want = copy.deepcopy(case['result'])
        if case['kind']=='wrapper':
            want['quality_restored'] = True
            want['factor_consumed'] = len(case['replies'])
            want['quality_consumed'] = len(case['qualities'])
        expected[case['name']] = want
    return '\n'.join(lines)+'\n', expected


def compare(txt, expected):
    lines = txt.splitlines()
    oracle.require(lines and lines[0]==HEADER, 'trace schema')
    oracle.equal(len(expected), len(lines)-1, 'trace case count')
    fields = 0
    def check(want, got, label):
        nonlocal fields
        oracle.require(type(want) is type(got), label+': type mismatch')
        if isinstance(want, dict):
            oracle.equal(set(want), set(got), label+'.keys')
            for k in want:
                check(want[k], got[k], label+'.'+k)
        elif isinstance(want, list):
            oracle.equal(len(want), len(got), label+'.length')
            for i,(w,g) in enumerate(zip(want,got)):
                check(w,g,f'{label}[{i}]')
        else:
            oracle.equal(want, got, label)
            fields += 1
    for line,(name,want) in zip(lines[1:], expected.items()):
        tag, body = line.split(';',1)
        oracle.equal(name, tag, 'trace case order')
        def no_duplicates(pairs):
            out = {}
            for k,v in pairs:
                oracle.require(k not in out, tag+'.duplicate '+k)
                out[k] = v
            return out
        check(want, json.loads(body, object_pairs_hook=no_duplicates), name)
    return dict(cases=len(expected), fields=fields, natural_select=28, natural_wrapper=6,
                synthetic_select=24, synthetic_wrapper=31,
                rule='ordered typed exact fields; pointer aliases; sentinel f64 bits')


def self_test(txt, expected):
    done = []
    def reject(name, lines):
        try:
            compare('\n'.join(lines)+'\n', expected)
        except (oracle.Fail, ValueError, TypeError, KeyError) as exc:
            done.append(dict(name=name, first_difference=str(exc)))
            return
        raise oracle.Fail('NOT REJECTED '+name)
    lines = txt.splitlines()
    reject('empty-targets', lines[:1])
    reject('missing-case', lines[:-1])
    reject('case-order', lines[:1]+[lines[2],lines[1]]+lines[3:])
    for case in ('synthetic/update', 'synthetic/repeat-two', 'synthetic/quality-request6', 'synthetic/return12'):
        index = list(expected).index(case)+1
        for key in expected[case]:
            altered = lines.copy()
            name, body = altered[index].split(';',1)
            value = json.loads(body)
            old = value[key]
            value[key] = not old if isinstance(old,bool) else old+1 if isinstance(old,int) else []
            altered[index] = name+';'+json.dumps(value)
            reject(case+'/'+key, altered)
        altered = lines.copy()
        altered[index] = altered[index][:-1]+',"request":999}'
        reject(case+'/duplicate', altered)
    return done


# Fixed expectations before mutation runs. Each must fail by an actual trace difference.
MUTATIONS = [
    ('limit-ge', 'input.updates == input.limit', 'input.updates >= input.limit'),
    ('overwrite-request', 'if request == 0 {', 'if true {'),
    ('limit-request10', 'request = 2;', 'request = 10;'),
    ('cost-ge-zero', 'cost > 0.0', 'cost >= 0.0'),
    ('cost-coefficient4', '(m * 5.0) * m', '(m * 4.0) * m'),
    ('sum-convert', '(input.l_nnz + input.u_nnz) as f64', '(input.l_nnz as f64 + input.u_nnz as f64)'),
    ('numeric-q-equal', 'input.updates > input.interval', 'input.updates >= input.interval'),
    ('ignore-numeric', 'input.numeric != 0', 'false'),
    ('always-call-support', 'request != 0 || input.support == 0', 'true'),
    ('negative-support-call', 'input.support == 0', 'input.support <= 0'),
    ('no-install-alpha', 'vectors.workspace = vectors.current;', 'vectors.workspace = saved;'),
    ('no-restore-alpha', 'vectors.workspace = saved;', 'vectors.workspace = vectors.current;'),
    ('swap-spike', 'vectors.workspace = vectors.current;', 'vectors.workspace = vectors.current; vectors.spike = vectors.current;'),
    ('no-repeat', 'if ret != 1 { break ret; }', 'if true { break ret; }'),
    ('return12-error', 'if ret == 12 { return FactorExit::Retry; }', 'if ret == 12 { return FactorExit::Error(ret); }'),
    ('status-before-error', 'if ret > 1 { return FactorExit::Error(ret); }', 'if ret > 1 { if state.status < -999 { state.status = 1; return FactorExit::Retry; } return FactorExit::Error(ret); }'),
    ('status-threshold', 'state.status < -999', 'state.status <= -999'),
    ('no-status-reset', 'state.status = 1;', 'state.status += 0;'),
    ('attempt-ignored', 'frame.attempt == 0 &&', 'true &&'),
    ('ignore-failure', 'state.failure != 0 ||', 'false ||'),
    ('numeric-gt', 'state.numeric >= frame.numeric_limit', 'state.numeric > frame.numeric_limit'),
    ('skip-recompute', 'if state.request != 0 { return FactorExit::Recompute; }', 'if false { return FactorExit::Recompute; }'),
    ('quality-gt', 'state.updates >= frame.period', 'state.updates > frame.period'),
    ('wrong-enter', 'frame.enter, frame.period, vectors', 'frame.enter + 1, frame.period, vectors'),
    ('quality-status-ignored', 'if state.status == 12 {', 'if false {'),
    ('quality-request-clear', 'if state.request != 0 {\n                frame.period = 20;', 'if false {\n                frame.period = 20;'),
    ('quality-period50', 'frame.period = 20;', 'frame.period = 50;'),
    ('quality-increment', 'frame.period += increment;', 'frame.period += increment + 1;'),
    ('clamp-zero-updates', 'state.updates != 0 && frame.period > 50', 'frame.period > 50'),
    ('no-period-clamp', 'frame.period = 50;', 'frame.period += 0;'),
]


def mutate_rust(path, expected):
    original = SOURCE.read_text()
    results = []
    try:
        for name, old, new in MUTATIONS:
            oracle.require(old in original, 'mutation target missing '+name)
            SOURCE.write_text(original.replace(old,new,1))
            proc = subprocess.run(['cargo','test','-p','rsolver-lp','--test','push_factor'], cwd=ROOT,
                                  text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
            if proc.returncode:
                # Exhausting replies can be an actual control error, but require a produced trace instead.
                results.append(dict(name=name, result='test-failed', log=proc.stdout))
                continue
            oracle.require(TRACE.exists(), name+': trace missing')
            try:
                compare(TRACE.read_text(), expected)
            except (oracle.Fail, ValueError, TypeError) as exc:
                results.append(dict(name=name, result='rejected', first_difference=str(exc)))
            else:
                results.append(dict(name=name, result='survived'))
    finally:
        SOURCE.write_text(original)
        restored = subprocess.run(['cargo','test','-p','rsolver-lp','--test','push_factor'], cwd=ROOT,
                                  text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        oracle.require(restored.returncode==0, restored.stdout)
        compare(TRACE.read_text(), expected)
        path.write_text(json.dumps(results,ensure_ascii=False,indent=2)+'\n')
    oracle.require(all(x['result']=='rejected' for x in results), 'mutation must reject by output: '+str([(x['name'],x['result']) for x in results if x['result']!='rejected']))
    return dict(rejected=len(results))


def manifest():
    files = ['crates/lp/src/crossover_factor.rs','crates/lp/src/lib.rs','crates/lp/tests/push_factor.rs',
             'tools/push_factor_rust.py','bench/oracle/m3_push_factor/manifest.json',
             'bench/oracle/m3_push_factor_rust/input.txt','bench/oracle/m3_push_factor_rust/trace.txt',
             'bench/oracle/m3_push_factor_rust/source.asm',
             'bench/oracle/m3_push_factor_rust/self-test.json','bench/oracle/m3_push_factor_rust/mutations.json',
             'bench/oracle/m3_push_factor_rust/oracle-regression.json','bench/oracle/m3_push_factor_rust/rust-regression.json']
    return dict(task='M3-04c-16', baseline='5358cf7', binary_sha256=oracle.BINARY_SHA,
                oracle_manifest_sha256=sha(ROOT/oracle.D/'manifest.json'),
                files={p:sha(ROOT/p) for p in files})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    g = parser.add_mutually_exclusive_group(required=True)
    for flag in ('write','compare','self-test','write-manifest'):
        g.add_argument('--'+flag, action='store_true')
    g.add_argument('--mutate-rust',type=Path)
    args = parser.parse_args()
    cs = cases()
    source, expected = project(cs)
    D.mkdir(parents=True,exist_ok=True)
    if args.write:
        INPUT.write_text(source)
        result = dict(written=len(cs))
    else:
        oracle.equal(source, INPUT.read_text(), 'fixed input projection')
        txt = TRACE.read_text()
        if args.mutate_rust:
            result = mutate_rust(args.mutate_rust, expected)
        else:
            result = compare(txt,expected)
            if args.self_test:
                tests = self_test(txt,expected)
                (D/'self-test.json').write_text(json.dumps(tests,ensure_ascii=False,indent=2)+'\n')
                result['rejected'] = len(tests)
            if args.write_manifest:
                (D/'trace.txt').write_text(txt)
                (D/'manifest.json').write_text(json.dumps(manifest(),ensure_ascii=False,indent=2)+'\n')
            (D/'comparison.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps(result,ensure_ascii=False))


if __name__=='__main__':
    try:
        main()
    except (oracle.Fail, KeyError, ValueError, TypeError) as exc:
        sys.exit('FAIL '+str(exc))
