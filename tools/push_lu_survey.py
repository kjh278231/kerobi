"""M3-04c-21: 검증된 저장 캡처의 수치 연결 경계를 집계한다.

새 수치 오라클이 아니다. 원본 실행/기준 갱신 없이 결정적 JSON을 출력한다.
--check는 저장 집계와 비교하고 --self-test는 입력 무결성 거부를 확인한다.
"""
import argparse
from collections import Counter
import copy
import hashlib
import json
from pathlib import Path

import push_round_check as prior

ROOT = Path(__file__).resolve().parents[1]
OUT = Path('bench/validation/m3_push_lu')
SOURCES = (
    '00120680_PRIVATE000000000009d008.c',
    '00120740_PRIVATE00000000002207de.c',
    '0012bd30_PRIVATE0000000000214344.c',
    '0012a990_PRIVATE000000000021b9de.c',
    '001223e0_PRIVATE000000000021686f.c',
    '001205d0_PRIVATE000000000009cf9b.c',
    '00131710_PRIVATE000000000009e376.c',
    '0012de60_PRIVATE0000000000221173.c',
    '000f15c0_PRIVATE000000000005b257.c',
    '001c0070_PRIVATE0000000000134a87.c',
    '001c5310_PRIVATE000000000025615c.c',
)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def count_values(rows, field):
    return dict(sorted(Counter(str(r[field]) for r in rows).items()))


def survey(name, record):
    events = record['events']
    select = lambda tag: [e for e in events if e['event'] == tag]
    if name == 'control':
        prior.equal([], events, 'control.events')
        return dict(name=name, events=0)
    ftrans = select('supplier:ftran:leave')
    btrans = select('roundctl:btran:leave')
    factors = select('factor:enter')
    prior.require(ftrans and btrans and factors, name + ': 필수 수치 경계0회')
    prior.equal(len(ftrans), len(select('supplier:ftran:enter')), name + '.ftran-pairs')
    prior.equal(len(btrans), len(select('roundctl:btran:enter')), name + '.btran-pairs')
    prior.equal(len(factors), len(select('factor:leave')), name + '.factor-pairs')
    for e in select('supply:enter'):
        prior.equal(0, e['batch'], name + '.batch')
    skipped = [e for e in select('factorctl:clear') if e['decision']]
    calls = [e for e in select('factorctl:call') if e['decision']]
    return dict(
        name=name, events=len(events),
        supplier_ftran=len(ftrans), raw_count=count_values([e['raw'] for e in ftrans], 'count'),
        spike_null=sum(e['spike'] is None for e in ftrans),
        spike_count=count_values([e['spike'] for e in ftrans if e['spike'] is not None], 'count'),
        support_btran=len(btrans), btran_count=count_values([e['raw'] for e in btrans], 'count'),
        approved=count_values(select('roundctl:accepted'), 'kind'),
        pivot_factor_calls=len(calls), pivot_factor_skips=len(skipped),
        factor_calls=len(factors), requests=count_values(factors, 'request'),
        factor_returns=count_values(select('factor:leave'), 'ret'),
        max_factor_updates=max(e['factor_ctl']['state']['k'] for e in factors),
        header_reordered=sum(a['header'] != b['header'] for a, b in zip(factors, select('factor:leave'))),
        recompute_ftran=len(select('recompute:ftran:leave')),
        recompute_sites=count_values(select('recompute:ftran:leave'), 'site'),
        observations=dict(input_after_supplier=False, lr_stage=False, u_kernel=False,
                          factor_arrays=False, buffer_swap_identity=False, btran_eta_scratch=False),
    )


def validate(data, manifest):
    prior.check_files(manifest)
    prior.controls(data, manifest)
    prior.acceptance(data)
    return [survey(n, data[n][0]) for n in ('arms', 'retry', 'interior', 'control')]


def report(data, manifest):
    rows = validate(data, manifest)
    natural = rows[:-1]
    binary = ROOT.parent / '.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so'
    prior.equal(prior.BINARY_SHA, sha(binary), 'local.binary_sha256')
    files = {f'../re/decomp/c/{p}': sha(ROOT.parent / 're/decomp/c' / p) for p in SOURCES}
    for p in ('crates/lp/src/lu.rs', 'crates/lp/src/simplex.rs',
              'docs/m1/m1_02i_eta.md', 'docs/m1/m1_02l_eta_peel.md',
              'docs/m1/m1_02m_usolve.md', 'docs/m1/m1_02n_update.md'):
        files[p] = sha(ROOT / p)
    assembly = sorted((ROOT / OUT).glob('*.asm'))
    prior.equal({'ftran-wrappers.asm', 'ftran-body.asm', 'btran-wrapper.asm',
                 'btran-body.asm', 'factor-renumber.asm'}, {p.name for p in assembly}, 'assembly.files')
    return dict(task='M3-04c-21', kind='stored-capture-survey', new_original_runs=0,
                numerical_equivalence='unverified', binary_sha256=prior.BINARY_SHA,
                survey_tool_sha256=sha(Path(__file__)), sources=files,
                assembly={str(p.relative_to(ROOT)): sha(p) for p in assembly},
                upstream_manifest_sha256=sha(ROOT / prior.D / 'manifest.json'),
                representative_captures={str(prior.D / f'{n}-1.json'): sha(ROOT / prior.D / f'{n}-1.json')
                                         for n in ('arms', 'retry', 'interior', 'control')},
                totals={k: sum(r[k] for r in natural) for k in
                        ('events', 'supplier_ftran', 'spike_null', 'support_btran', 'pivot_factor_calls',
                         'pivot_factor_skips', 'factor_calls', 'header_reordered', 'recompute_ftran')},
                datasets=rows)


def self_test(data, manifest):
    rejected = []

    def reject(name, mutate):
        d, m = copy.deepcopy(data), copy.deepcopy(manifest)
        mutate(d, m)
        try:
            validate(d, m)
        except (prior.Fail, KeyError, TypeError, IndexError, ValueError) as exc:
            rejected.append(dict(name=name, error=str(exc)))
            return
        raise prior.Fail('NOT REJECTED ' + name)

    reject('binary-zero', lambda d, m: d['arms'][0].update(binary_sha256='0' * 64))
    reject('model-zero', lambda d, m: d['arms'][0].update(model_sha256='0' * 64))
    reject('empty-events', lambda d, m: d['arms'][0].update(events=[]))
    for tag in ('supplier:ftran:', 'roundctl:btran:', 'factor:', 'recompute:'):
        def remove(d, m, tag=tag):
            for rec in d['arms']:
                rec['events'] = [e for e in rec['events'] if not e['event'].startswith(tag)]
        reject('remove-' + tag, remove)
    reject('spike-field-missing', lambda d, m:
           next(e for e in d['interior'][0]['events'] if e['event'] == 'supplier:ftran:leave').pop('spike'))
    return dict(kind='input-integrity-only', rejected=rejected)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--self-test', action='store_true')
    args = parser.parse_args()
    data, manifest = prior.records(), prior.load(prior.D / 'manifest.json')
    result = report(data, manifest)
    if args.check:
        prior.equal(json.loads((ROOT / OUT / 'survey.json').read_text()), result, 'survey')
    if args.self_test:
        print(json.dumps(self_test(data, manifest), ensure_ascii=False, indent=2))
    elif args.check:
        print(json.dumps(result['totals'], ensure_ascii=False, sort_keys=True))
    else:
        print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    main()
