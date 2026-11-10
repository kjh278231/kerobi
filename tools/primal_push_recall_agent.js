// M3-04c-8: M3-04c-7 재시도 agent의 단계 frame에 local_1d8(0x1f00(%rsp), theta=I 재호출의 자유 열 판정과 flip 상태가 읽는 -I 후보)만 더했다. 호출과 메모리를 대체하지 않는다.
Process.attachModuleObserver({onAdded(mod) {
  if (mod.name !== 'libgurobi130.so') return;
  const b = mod.base;
  let active = null, calls = 0;
  const hex = (p, size) => Array.from(new Uint8Array(p.readByteArray(size))).map(x => x.toString(16).padStart(2, '0')).join('');
  function arr(p, n, size) {
    if (n < 0 || n > 100000) throw Error('배열 길이 오류: ' + n);
    if (p.isNull()) throw Error('필수 배열 null');
    return Array.from({length:n}, (_, i) => size === 4 ? p.add(i*size).readS32() : hex(p.add(i*size), size));
  }
  const vec = (s, off, n, size=8) => arr(s.add(off).readPointer(), n, size);
  function sparse(p, m) {
    const count = p.readS32(), n = count < 0 ? m : count;
    return {count, indices:count < 0 ? null:vec(p,8,n,4), values:vec(p,16,n)};
  }
  function snap(detail=false) {
    const a=active, s=a.s, m=s.add(0x64).readS32(), n=s.add(0x68).readS32(), f=s.add(0x470).readPointer();
    const r={m,n,status:s.add(0xf0).readS32(),precision:s.add(0x28).readS32(),
      vstat:vec(s,0x168,n+m,4),header:vec(f,0x18,m,4),x:arr(a.x,n,8),slack:arr(a.slack,m,8),
      factor:{updates:f.add(0x18c).readS32(),limit:f.add(0x188).readS32(),nnzL:hex(f.add(0x1e0),8),nnzU:hex(f.add(0x1e8),8)},
      counters:{numeric:s.add(0x178).readS32(),failure:s.add(0x184).readS32(),request:s.add(0x294).readS32(),iterations:hex(s.add(0x2a0),8)},
      tolerances:Object.fromEntries([0x370,0x380,0x390,0x398,0x3b0].map(off=>[off.toString(16),hex(s.add(off),8)]))};
    if(detail) Object.assign(r,{basic:vec(s,0x1e8,m),width:vec(s.add(0x208).readPointer(),0x10,m),marks:vec(s,0x280,m,1),rhs:vec(s,0x200,m),pinf:hex(s.add(0x120),8)});
    return r;
  }
  function emit(event, fields={}) {send({event,call:active.call,step:active.step,...fields});}
  function selected() {const s=active.s; return {enter:s.add(0x2fc).readS32(),leave:s.add(0x2f8).readS32(),direction:s.add(0x328).readS32()};}
  function finish() {
    if(active.pending) {emit('step:leave',{kind:active.pending,state:snap(true),selected:selected()});active.pending=null;active.applied=false;}
  }
  Interceptor.attach(b.add(0x1c0070),{
    onEnter(a) {
      if(active) throw Error('push 중첩');
      active={call:++calls,step:0,s:a[0],x:a[3],slack:a[4],condition:a[6],retry:a[7],remaining:a[8],pending:null};
      const s=a[0], n=s.add(0x68).readS32(), m=s.add(0x64).readS32(), starts=vec(s,0x70,n,8), lengths=vec(s,0x78,n,4);
      // CSC start는 64비트 정수이며 주소가 아니다.
      let nnz=0;for(let j=0;j<n;j++) nnz=Math.max(nnz,Number(s.add(0x70).readPointer().add(j*8).readS64())+lengths[j]);
      emit('push:enter',{first:a[5].toInt32(),model_status:a[1].toInt32(),attempt:a[2].toInt32(),alias_contiguous:a[4].equals(a[3].add(n*8)),
        model:{starts,lengths,rows:vec(s,0x80,nnz,4),values:vec(s,0x88,nnz),cost:vec(s,0x98,n+m),rhs:vec(s,0xa0,m),lb:vec(s,0xa8,n+m),ub:vec(s,0xb0,n+m)},state:snap()});
    },
    onLeave(r) {
      finish();emit('push:leave',{ret:r.toInt32(),condition:hex(active.condition,8),retry:active.retry.readS32(),remaining:active.remaining.readS32(),state:snap(true)});active=null;
    }
  });
  Interceptor.attach(b.add(0x2937b0),{onEnter(){send({event:'unsupported:x87'});}});
  Interceptor.attach(b.add(0x1c5310),{
    onEnter(a) {
      this.enabled=active!==null;if(!this.enabled)return;
      finish();this.local=a[0];this.out=a[2];
      const p=this.local, total=p.add(4).readS32(), s=active.s, nm=s.add(0x64).readS32()+s.add(0x68).readS32();
      emit('supply:enter',{candidates:vec(p,8,total,4),position:p.add(0x20).readS32(),failures:vec(p,0x10,nm,4),support:a[3].readS32(),support_marks:a[3].readS32()>0?vec(p,0x18,s.add(0x64).readS32(),4):null,batch:p.add(0x1840).readS32(),state:snap(true)});
    },
    onLeave(r) {
      if(!this.enabled)return;
      const candidate=r.toInt32();if(candidate>=0)active.step++;
      emit('supply:leave',{candidate,position:this.local.add(0x20).readS32(),alpha:candidate<0?null:sparse(this.out.readPointer(),active.s.add(0x64).readS32())});
    }
  });
  for(const [name,off,feasible] of [['feasible',0xef960,true],['infeasible',0xeefa0,false]]) Interceptor.attach(b.add(off),{
    onEnter(a) {
      this.enabled=active!==null;if(!this.enabled)return;
      const m=active.s.add(0x64).readS32();
      this.theta=a[feasible?11:8];this.pivot=a[feasible?12:9];this.work=a[feasible?13:10];
      this.alpha=sparse(a[feasible?5:2],m);
      emit('ratio:enter',{name,selected:selected(),alpha:this.alpha,basic:arr(a[feasible?6:3],m,8),width:arr(a[feasible?7:4],m,8),marks:arr(a[feasible?8:5],m,1),state:snap(true)});
    },
    onLeave(r) {
      if(!this.enabled)return;
      const chosen=r.toInt32(),row=chosen<0?chosen:this.alpha.count<0?chosen:this.alpha.indices[chosen];
      emit('ratio:leave',{name,chosen,row,theta:hex(this.theta,8),pivot:hex(this.pivot,8),work:this.work.isNull()?null:hex(this.work,8)});
    }
  });
  // 위반 basis에서 054eef가 -1 또는 |pivot|<0.001이면 push가 부르는 재시도 ratio. 인자: rdi=S, rsi=m, rdx=alpha, rcx=basic,
  // r8=width, r9=marks, 스택 theta*·pivot*·scratch 2개·work*(0x1c25b4-0x1c263a).
  Interceptor.attach(b.add(0xf0120),{
    onEnter(a) {
      this.enabled=active!==null;if(!this.enabled)return;
      if(!a[0].equals(active.s))throw Error('재시도 workspace 불일치');
      const m=active.s.add(0x64).readS32();
      this.theta=a[6];this.pivot=a[7];this.work=a[10];this.alpha=sparse(a[2],m);
      emit('retry:enter',{selected:selected(),theta:hex(this.theta,8),pivot:hex(this.pivot,8),alpha:this.alpha,basic:arr(a[3],m,8),width:arr(a[4],m,8),
        marks:arr(a[5],m,1),work:this.work.isNull()?null:hex(this.work,8),factor_updates:active.s.add(0x470).readPointer().add(0x18c).readS32(),work_coef:hex(active.s.add(0x148),8)});
    },
    onLeave(r) {
      if(!this.enabled)return;
      const chosen=r.toInt32(),row=chosen<0?chosen:this.alpha.count<0?chosen:this.alpha.indices[chosen];
      emit('retry:leave',{chosen,row,theta:hex(this.theta,8),pivot:hex(this.pivot,8),work:this.work.isNull()?null:hex(this.work,8),
        request:active.s.add(0x294).readS32(),leave:active.s.add(0x2f8).readS32()});
    }
  });
  for(const [kind,off] of [['flip',0x1c2994],['pivot',0x1c2ab5]]) Interceptor.attach(b.add(off),{
    onEnter() {
      if(!active)throw Error('push 밖의 단계');
      if(!this.context.r14.equals(active.s))throw Error('workspace ABI 불일치');
      if(active.pending)throw Error('겹친 단계');
      active.pending=kind;emit('step:enter',{kind,selected:selected(),theta:hex(this.context.rsp.add(0x1eb8),8),pivot:kind==='pivot'?hex(this.context.rsp.add(0x1ef8),8):null,state:snap(true),
        // r·dL(ratio 호출 전후 보존 슬롯), dU, 100h, -100h, 2h, 0.01h, local_1d8(0x1c497e·0x1c4d55·0x1c29ac가 읽는 슬롯)
        frame:Object.fromEntries([['r',0x1ed0],['dL',0x1ed8],['dU',0x1ee0],['k100',0x1ec8],['km100',0x1eb0],['k2',0x1e88],['e',0x2080],['ninf',0x1f00]].map(([k,o])=>[k,hex(this.context.rsp.add(o),8)]))});
    }
  });
  // 보류 지점. 들어오는 경로는 승인 검사의 ja 두 개(0x1c2a99 pivot<1e-6, 0x1c2aaf theta*pivot<-1e-5)뿐이고 직전 명령은 무조건 jmp다.
  Interceptor.attach(b.add(0x1c3b7b),{
    onEnter() {
      if(!active)throw Error('push 밖의 보류');
      if(!this.context.r14.equals(active.s))throw Error('workspace ABI 불일치');
      if(active.pending)throw Error('겹친 단계');
      const sp=this.context.rsp;
      emit('step:defer',{selected:selected(),theta:hex(sp.add(0x1eb8),8),pivot:hex(sp.add(0x1ef8),8),deferred:sp.add(0x2c).readS32(),state:snap(true),
        frame:Object.fromEntries([['r',0x1ed0],['dL',0x1ed8],['dU',0x1ee0],['k100',0x1ec8],['km100',0x1eb0],['k2',0x1e88],['e',0x2080],['ninf',0x1f00]].map(([k,o])=>[k,hex(sp.add(o),8)]))});
    }
  });
  // flip·pivot 경로가 합류하는 교환 직후·rhs 보정 전 지점이다.
  Interceptor.attach(b.add(0x1c3004),{
    onEnter() {
      if(!active||!active.pending)throw Error('적용 단계 연결 오류');
      if(!this.context.r14.equals(active.s))throw Error('workspace ABI 불일치');
      if(active.applied)throw Error('겹친 적용');
      active.applied=true;emit('step:applied',{kind:active.pending,selected:selected(),state:snap(true)});
    }
  });
  Interceptor.attach(b.add(0xf15c0),{
    onEnter(a){this.enabled=active!==null;if(this.enabled)emit('factor:enter',{request:active.s.add(0x294).readS32(),header:vec(active.s.add(0x470).readPointer(),0x18,active.s.add(0x64).readS32(),4)});},
    onLeave(r){if(this.enabled)emit('factor:leave',{ret:r.toInt32(),request:active.s.add(0x294).readS32(),header:vec(active.s.add(0x470).readPointer(),0x18,active.s.add(0x64).readS32(),4)});}
  });
}});
