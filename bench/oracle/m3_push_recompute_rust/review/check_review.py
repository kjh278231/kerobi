"""M3-04c-18 독립 검토: 저장 재생·재캡처·오류 주입. 구현 파일은 수정하지 않는다."""
import copy
import hashlib
import json
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[4]/'tools'))
import push_recompute_check as oracle
import push_recompute_extra as extra
import push_recompute_rust as rust

ROOT=rust.ROOT
HERE=Path(__file__).resolve().parent

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()

def main():
    original=oracle.records()
    manifest=oracle.load(oracle.D/'manifest.json')
    oracle.check_files(manifest)
    controls=oracle.controls(original,manifest)
    acceptance=oracle.acceptance(original)
    oracle_errors=oracle.self_test(original,manifest)
    data=extra.load_checked()
    m=json.loads((extra.D/'extra-manifest.json').read_text())
    extra_errors=extra.self_test(data,m)
    recap=json.loads((HERE/'extra.json').read_text())
    base=data[0]
    oracle.equal(set(base),set(recap),'independent recapture schema keys')
    for key in base:
        if key not in ('pid','run_id'):
            oracle.equal(base[key],recap[key],'independent recapture.'+key)
    oracle.require(recap['pid'] not in {r['pid'] for r in data},'independent new pid')
    oracle.require(recap['run_id'] not in {r['run_id'] for r in data},'independent new run id')
    cs=rust.cases(); source,expected=rust.project(cs); header=source.splitlines()[0]
    oracle.equal(source,rust.INPUT.read_text(),'independent fixed inputs')
    trace=rust.TRACE.read_text()
    compared=rust.compare(trace,expected,header)
    trace_errors=rust.self_test(trace,expected,header)
    direct=[]
    def reject(name,fn):
        try:fn()
        except (oracle.Fail,KeyError,TypeError,ValueError,IndexError) as exc:
            direct.append(dict(name=name,first_difference=str(exc)));return
        raise AssertionError('직접 변조를 거부하지 못함: '+name)
    def altered_capture(key,value):
        d=copy.deepcopy(data);d[0][key]=value;extra.check(d,m)
    reject('capture-binary-zero',lambda:altered_capture('binary_sha256','0'*64))
    reject('capture-input-zero',lambda:altered_capture('input_sha256','0'*64))
    reject('capture-empty-targets',lambda:extra.check([dict(r,events=[]) for r in data],m))
    reject('capture-manifest-zero',lambda:extra.check(data,dict(m,binary_sha256='0'*64)))
    def altered_result(key,value):
        d=copy.deepcopy(data)
        for r in d:r['events'][3]['result'][key]=value
        extra.check(d,m)
    reject('capture-null-result',lambda:extra.check([dict(r,events=[dict(e,result=None) for e in r['events']]) for r in data],m))
    reject('capture-association-pinf-four',lambda:altered_result('pinf',extra.native.bits(4.)))
    reject('capture-classification',lambda:altered_result('marks',['4e','4e']))
    reject('capture-request-cleared',lambda:altered_result('request',0))
    reject('capture-alias-change',lambda:altered_result('aliases',dict(rhs_input=False,raw_basic=True)))
    lines=trace.splitlines()
    def altered_trace(key,value):
        changed=lines.copy();tag,body=changed[1].split(';',1)
        v=json.loads(body);v[key]=value
        changed[1]=tag+';'+json.dumps(v)
        rust.compare('\n'.join(changed)+'\n',expected,header)
    reject('trace-pinf-one-bit',lambda:altered_trace('pinf','0100000000000000'))
    reject('trace-classification',lambda:altered_trace('marks',['4e']*len(expected[next(iter(expected))]['marks'])))
    reject('trace-request-cleared',lambda:altered_trace('request',0))
    reject('trace-typed-bool',lambda:altered_trace('violations',False))
    reject('trace-empty-targets',lambda:rust.compare(header+'\n',expected,header))
    reject('trace-null-result',lambda:rust.compare('\n'.join([header,lines[1].split(';',1)[0]+';null',*lines[2:]])+'\n',expected,header))
    reject('trace-schema-hash-zero',lambda:rust.compare('\n'.join([rust.SCHEMA+' '+'0'*64,*lines[1:]])+'\n',expected,header))
    mutation=json.loads((rust.D/'mutations.json').read_text())
    oracle.equal([x[0] for x in rust.MUTATIONS],[x['name'] for x in mutation],'parent Rust mutation names/order')
    oracle.equal({'rejected'},{x['result'] for x in mutation},'parent Rust mutation results')
    watched=['crates/lp/src/crossover_recompute.rs','crates/lp/tests/push_recompute.rs','tools/push_recompute_rust.py','tools/push_recompute_extra.py','tools/push_recompute_synthetic.py','bench/oracle/m3_push_recompute/source.asm','bench/oracle/m3_push_recompute/manifest.json','bench/oracle/m3_push_recompute_rust/extra-manifest.json','bench/oracle/m3_push_recompute_rust/input.txt','target/m3_push_recompute_rust/trace.txt','bench/oracle/m3_push_recompute_rust/mutations.json']
    out=dict(task='M3-04c-18',reviewer='/root/m3_04c18_review',verdict='계약 범위 통과',
        recapture=dict(command='tools/gurun.sh ../.venv/bin/python tools/push_recompute_extra.py --capture bench/oracle/m3_push_recompute_rust/review/extra.json',cases=10,rows=18,all_nonidentity_fields_equal=True,pid=recap['pid'],run_id=recap['run_id'],sha256=sha(HERE/'extra.json'),binary_sha256=recap['binary_sha256']),
        original_controls=controls,original_acceptance=acceptance,comparison=compared,
        error_detection=dict(original_self_test=oracle_errors['rejected'],extra_self_test=len(extra_errors),trace_self_test=len(trace_errors),direct_rejected=len(direct),direct_details=direct),
        parent_rust_mutations=dict(count=len(mutation),rejected=len(mutation),fresh_cargo_execution=False),
        findings=[dict(severity='낮음',status='수정 확인',file='crates/lp/src/crossover_recompute.rs',description='최종 주석이 bounds의 열 좌표 길이와 basic/width/marks의 basis 길이를 구분함을 확인했다.'),dict(severity='정리중',file='docs/m3/primal_push_recompute_rust.md',description='계약의 초기128사례·추가 원본 실행 불필요·변이4생존·독립검토 미사용 설명은 최종138사례 결과와 함께 갱신할 필요가 있다. 부모에게 전달함.')],
        limitations=['유한 입력과 유효 header/bounds, retry 초기0 범위','별칭true·free/fixed·retry·정확경계는 자연 도달성을 검증하지 않은 합성 증거','rhs/FTRAN 호출·수치 backend, 실제LU21·루프22·batch23·x87·전체work/P3는 제외','검토자는 Cargo 재빌드·Rust 소스변이를 새로 실행하지 않고 부모의33종 출력 차이를 검토함'],
        hashes={p:sha(ROOT/p) for p in watched},source_c_sha256=sha(ROOT/'../re/decomp/c/001c0070_PRIVATE0000000000134a87.c'))
    (HERE/'review.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps(dict(verdict=out['verdict'],recaptured_cases=10,compared=compared,direct_rejected=len(direct),original_self_test=oracle_errors['rejected'],extra_self_test=len(extra_errors),trace_self_test=len(trace_errors)),ensure_ascii=False))

if __name__=='__main__':main()
