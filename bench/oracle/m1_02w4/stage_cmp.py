"""M1-02w-4 exploration: compare the original e_r stage capture (tools/m1_er_dense.js) with the Rust probe
(RSOLVER_BTRAN_PROBE=<spec> RSOLVER_STAGE_PROBE=k0,k1) call by call: row etas, U^T output (set, order, value bits), the
dense vector after the etas and the final output. Original indices are basis positions; mapped with the probe pos_row.
usage: python3 bench/oracle/m1_02w4/stage_cmp.py ORIGINAL.json.gz RUST_SPEC.out"""
import gzip,json,sys
o=json.load(gzip.open(sys.argv[1]))
ev={e[0]:e for e in o['events']}
import gzip as _g
rs=[json.loads(l) for l in (_g.open(sys.argv[2], 'rt') if sys.argv[2].endswith('.gz') else open(sys.argv[2]))]
nat={x['k']:x for x in rs if 'r' in x and 'k' in x and 'i' not in x}
st={x['stage']:x for x in rs if 'stage' in x}
def ulp(a,b):
    a=int(a,16); b=int(b,16)
    return abs(a-b) if (a>>63)==(b>>63) else 'sign'
bad_total = 0
for k in sorted(st):
    e=ev[k]; d=e[4]; s=st[k]; n=nat[k]; pr=s['pos_row']
    print(f"k {k} r o{e[1]}/R{n['r']} etas o{e[2]}/R{n['e']} hit912 {d['hit912']} out_nnz {e[3]}")
    oe=d['etas']; re_=s['etas']; bad=0
    for i,(a,b) in enumerate(zip(oe,re_)):
        ai=[pr[x] for x in a[1]]
        if pr[a[0]]!=b[0] or ai!=b[1][0]:
            print('  eta',i,'idx differs r',pr[a[0]],b[0], sorted(ai)==sorted(b[1][0])); bad+=1; continue
        for j,(x,y) in enumerate(zip(a[2],b[1][1])):
            if x!=y: print('  eta',i,'pos',j,'row',ai[j],x,y,ulp(x,y)); bad+=1
    print('  etas compared',len(oe),'bad',bad)
    on,oi,ov=d['ut']; ri,rv=s['ut']
    oi=[pr[x] for x in oi]
    od=dict(zip(oi,ov)); rd=dict(zip(ri,rv))
    print('  ut nnz',on,len(ri),'order same',oi==ri,'set same',set(oi)==set(ri))
    for i in sorted(set(oi)|set(rd)):
        if od.get(i)!=rd.get(i): print('   ut diff row',i,od.get(i),rd.get(i))
    eo=d['eta_out']
    if eo is None: print('  no eta_out'); continue
    re2=s['eta_out']
    # original eta_out indexed by position; map to row
    eor=[None]*len(eo)
    for p,v in enumerate(eo): eor[pr[p]]=v
    diffs=[i for i in range(len(eo)) if eor[i]!=re2[i]]
    print('  eta_out diffs',len(diffs))
    bad += len(diffs)
    for i in diffs[:6]: print('    row',i,eor[i],re2[i],ulp(eor[i],re2[i]))
    diffs=[i for i,(x,y) in enumerate(zip(d['out_val'],n['val'])) if x!=y]
    print('  out diffs',len(diffs), diffs[:8])
    bad_total += bad + len(diffs) + (oi != ri) + sum(od.get(i) != rd.get(i) for i in set(oi) | set(rd))

print('TOTAL DIFFS', bad_total)
sys.exit(1 if bad_total else 0)
