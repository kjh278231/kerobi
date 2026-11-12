"""M3-04c-12: 기존 자연 캡처의 루프 사건을 집계한다. 새 동등성 오라클이 아니다.

선행 검사기로 출처·독립 3회·대조군을 재검사한 각 사례의 첫 기록만 집계한다.
표준 출력은 결정적 JSON이다. 원본을 실행하거나 기준 자료를 수정하지 않는다.
"""
from collections import Counter
import hashlib
import json
from pathlib import Path

import primal_push_decision_check as dc
import push_chain_oracle as chain

ROOT = Path(__file__).resolve().parents[1]
SOURCES = (
    '001c0070_PRIVATE0000000000134a87.c',
    '001c5310_PRIVATE000000000025615c.c',
    '000f15c0_PRIVATE000000000005b257.c',
    '00115ed0_PRIVATE0000000000089f61.c',
    '001205c0_PRIVATE000000000009c062.c',
    '00120260_PRIVATE000000000009bae6.c',
)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def datasets():
    manifest = json.loads((dc.BASE / 'manifest.json').read_text())
    records = dc.load_all(manifest, arms=True, violated=True, retry=True, recall=True)
    cases = chain.all_cases()
    chain.check(cases, json.loads((ROOT / chain.D / 'manifest.json').read_text()))
    records += [(f'chain-{lp}-{kind}', recs[0]) for (lp, kind), recs in cases.items()]
    return records


def survey(label, rec):
    events = rec['events']
    counts = Counter(e['event'] for e in events)
    supply = [e for e in events if e['event'] == 'supply:enter']
    chosen = [e for e in events if e['event'] == 'supply:leave' and e['candidate'] >= 0]
    assert supply and chosen, (label, '공급 사건 없음')
    assert counts['supply:enter'] == counts['supply:leave'], (label, '공급 경계 누락')
    assert counts['factor:enter'] == counts['factor:leave'], (label, 'factor 경계 누락')
    last_supply = None
    factor_rows = []
    for i, e in enumerate(events):
        if e['event'] == 'supply:leave':
            last_supply = e['candidate']
        elif e['event'] == 'factor:enter':
            leave = events[i + 1]
            assert leave['event'] == 'factor:leave', (label, 'factor 중첩·미완결')
            factor_rows.append(dict(call=e['call'], step=e['step'], request=e['request'],
                                    ret=leave['ret'], request_after=leave['request'],
                                    context='initial' if last_supply is None else
                                    'after-exhaustion' if last_supply < 0 else 'after-candidate',
                                    header_reordered=e['header'] != leave['header']))
    return dict(label=label, events=len(events),
                events_sha256=hashlib.sha256(json.dumps(events, sort_keys=True).encode()).hexdigest(),
                dimensions=[supply[0]['state']['m'], supply[0]['state']['n']],
                supplied=len(chosen), exhausted=counts['supply:leave'] - len(chosen),
                support_modes=dict(sorted(Counter(str(e['support']) for e in supply).items())),
                max_failures=max(max(e['failures'], default=0) for e in supply),
                batch_capacities=sorted({e['batch'] for e in supply}),
                dense_outputs=sum(e['alpha']['count'] < 0 for e in chosen),
                defer=counts['step:defer'], factors=factor_rows,
                remaining=[e['remaining'] for e in events if e['event'] == 'push:leave'])


def main():
    rows = [survey(label, rec) for label, rec in datasets()]
    factors = [f for r in rows for f in r['factors']]
    sources = {f'../re/decomp/c/{name}': sha(ROOT.parent / 're/decomp/c' / name) for name in SOURCES}
    report = dict(task='M3-04c-12', kind='stored-capture-survey', new_original_runs=0,
                  implementation_equivalence='unverified', binary_sha256=chain.BINARY_SHA,
                  sources=sources, survey_tool_sha256=sha(Path(__file__)),
                  totals=dict(datasets=len(rows), supplied=sum(r['supplied'] for r in rows),
                              exhausted=sum(r['exhausted'] for r in rows),
                              defer=sum(r['defer'] for r in rows), factor_calls=len(factors),
                              requests=dict(sorted(Counter(str(f['request']) for f in factors).items())),
                              returns=dict(sorted(Counter(str(f['ret']) for f in factors).items())),
                              header_reordered=sum(f['header_reordered'] for f in factors),
                              dense_outputs=sum(r['dense_outputs'] for r in rows),
                              batch_capacities=sorted({b for r in rows for b in r['batch_capacities']})),
                  datasets=rows)
    print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    main()
