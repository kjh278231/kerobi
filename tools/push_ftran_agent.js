// M3-04c-19: 선행17 비간섭 + 보류/지원/라운드 읽기 전용 관측.
// M3-04c-17: 선행15 사건 보존, 두 재계산 경계 읽기 전용 관측.
// M3-04c-15: 선행13 사건을 보존하고 직렬 factor caller 관측을 추가한다.
// M3-04c-13: 공급자 P 전후와 중첩 FTRAN의 원본 열·raw 출력·spike 별칭을 추가한다.
// 기반 M3-04c-8: M3-04c-7 재시도 agent의 단계 frame에 local_1d8(0x1f00(%rsp), theta=I 재호출의 자유 열 판정과 flip 상태가 읽는 -I 후보)만 더했다. 호출과 메모리를 대체하지 않는다.
// M3-04c-21a: 선행19 사건 보존 + 한 RHS FTRAN 본체 2207de·L/U 커널 읽기 전용 관측.
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
  let supplier = null, supplierCalls = 0, ftClearList = null;
  function contextState(p, support, m, nm) {
    const total=p.add(4).readS32(), sp=support.readS32();
    return {m:p.readS32(),total,candidates:vec(p,8,total,4),position:p.add(0x20).readS32(),
      deferred:p.add(0x24).readS32(),failures:vec(p,0x10,nm,4),support:sp,
      // support<=0이면 원본이 marks를 읽지 않는다. 초기화되지 않은 메모리를 직렬화하지 않는다.
      support_marks:sp>0?vec(p,0x18,m,4):null,count_sum:hex(p.add(0x30),8),successes:p.add(0x38).readS32(),
      batch:[0x1840,0x1844,0x1848].map(o=>p.add(o).readS32())};
  }
  function spikeState(p,m) {
    if(p.isNull())return null;
    const count=p.readS32(), indices=count<0?null:vec(p,8,count,4), values=p.add(16).readPointer();
    return {count,indices,values:count<0?arr(values,m,8):indices.map(r=>hex(values.add(r*8),8))};
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
      active.attempt=a[2].toInt32();active.numeric_limit=a[0].add(0x178).readS32()+10;
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
      finish();this.local=a[0];this.out=a[2];this.support=a[3];
      active.roundSp=a[0].sub(8);
      if(a[3].readS32()>0)active.marksKnown=true;
      if(active.roundDefer){roundEmit('defer:leave',active.roundSp);active.roundDefer=false;}
      if(active.passStart){roundEmit('pass:ready',active.roundSp);active.passStart=false;}
      if(active.roundClear){roundEmit('clear',active.roundSp);active.roundClear=false;}

      if(supplier)throw Error("공급자 중첩");
      supplier={id:++supplierCalls,work:a[4]};
      const p=this.local, total=p.add(4).readS32(), s=active.s, nm=s.add(0x64).readS32()+s.add(0x68).readS32();
      emit('supply:enter',{candidates:vec(p,8,total,4),position:p.add(0x20).readS32(),failures:vec(p,0x10,nm,4),support:a[3].readS32(),support_marks:a[3].readS32()>0?vec(p,0x18,s.add(0x64).readS32(),4):null,batch:p.add(0x1840).readS32(),state:snap(true),supplier:{id:supplier.id,context:contextState(p,a[3],s.add(0x64).readS32(),nm),drop:hex(s.add(0x3a8),8)}});
    },
    onLeave(r) {
      if(!this.enabled)return;
      const candidate=r.toInt32();if(candidate>=0){active.step++;active.passFresh=false;}
      emit('supply:leave',{candidate,position:this.local.add(0x20).readS32(),alpha:candidate<0?null:sparse(this.out.readPointer(),active.s.add(0x64).readS32()),supplier:{id:supplier.id,context:contextState(this.local,this.support,active.s.add(0x64).readS32(),active.s.add(0x64).readS32()+active.s.add(0x68).readS32()),output_is_s210:this.out.readPointer().equals(active.s.add(0x210).readPointer())}});
      supplier=null;
    }
  });
  Interceptor.attach(b.add(0x120680),{
    onEnter(a) {
      this.enabled=supplier!==null;if(!this.enabled)return;
      const s=active.s, f=s.add(0x470).readPointer(), m=s.add(0x64).readS32();
      this.m=m;this.input=a[1];this.output=a[2];this.spike=a[3];this.work=a[7];
      // M3-04c-21a: 지우기 전 spike 목록만 보관한다(사건 변경 없음).
      ftClearList=a[3].isNull()?null:spikeState(a[3],m);
      emit('supplier:ftran:enter',{supplier_id:supplier.id,input:sparse(a[1],m),spike:spikeState(a[3],m),
        aliases:{factor:a[0].equals(f),input_s220:a[1].equals(s.add(0x220).readPointer()),output_s210:a[2].equals(s.add(0x210).readPointer()),
          spike_null:a[3].isNull(),spike_f1c8:a[3].equals(f.add(0x1c8).readPointer()),work:a[7].equals(supplier.work),
          zeros:[a[4],a[5],a[6]].map(x=>x.isNull())},work:this.work.isNull()?null:hex(this.work,8)});
    },
    onLeave() {
      if(!this.enabled)return;
      emit('supplier:ftran:leave',{supplier_id:supplier.id,raw:sparse(this.output,this.m),spike:spikeState(this.spike,this.m),
        work:this.work.isNull()?null:hex(this.work,8)});
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
      roundEmit('defer:enter',sp);active.roundDefer=true;
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
      roundEmit('accepted',this.context.rsp,{kind:active.pending});
      active.applied=true;emit('step:applied',{kind:active.pending,selected:selected(),state:snap(true)});
    }
  });
  Interceptor.attach(b.add(0xf15c0),{
    onEnter(a){this.enabled=active!==null;if(!this.enabled)return;
      this.spike=active.s.add(0x470).readPointer().add(0x1c8).readPointer();
      emit('factor:enter',{request:active.s.add(0x294).readS32(),header:vec(active.s.add(0x470).readPointer(),0x18,active.s.add(0x64).readS32(),4),
        factor_ctl:{state:ctlState(),caller:this.returnAddress.sub(b).toString(),workspace:a[0].equals(active.s),
          alpha_is_current:active.frame?active.s.add(0x210).readPointer().equals(active.frame.add(0x2090).readPointer()):null}});},
    onLeave(r){if(!this.enabled)return;emit('factor:leave',{ret:r.toInt32(),request:active.s.add(0x294).readS32(),header:vec(active.s.add(0x470).readPointer(),0x18,active.s.add(0x64).readS32(),4),
      factor_ctl:{state:ctlState(),spike_preserved:active.s.add(0x470).readPointer().add(0x1c8).readPointer().equals(this.spike)}});}
  });

  function ctlState() {
    const s=active.s,f=s.add(0x470).readPointer();
    return {m:s.add(0x64).readS32(),request:s.add(0x294).readS32(),numeric:s.add(0x178).readS32(),failure:s.add(0x184).readS32(),
      status:s.add(0xf0).readS32(),q:s.add(0x46c).readS32(),k:f.add(0x18c).readS32(),limit:f.add(0x188).readS32(),
      C:f.add(0x190).readS64().toString(),E:f.add(0x184).readS32(),L:f.add(0x1e0).readS64().toString(),U:f.add(0x1e8).readS64().toString(),
      quality_tol:hex(s.add(0x3a0),8),markowitz:hex(s.add(0x390),8)};
  }
  function frameState(sp) {
    return {support:sp.add(0x209c).readS32(),period:sp.add(0x1fd8).readS32(),attempt:sp.add(0x1fa0).readS32(),
      numeric_limit:sp.add(0x1fe0).readS32(),batch:[0x1848,0x184c,0x1850].map(o=>sp.add(o).readS32()),
      constants:[0x1e78,0x1e80,0x1e98].map(o=>hex(sp.add(o),8))};
  }
  function probe(off,fn) {Interceptor.attach(b.add(off),function(){
    if(!active||!this.context.r14.equals(active.s))throw Error('factor caller ABI');
    fn(this.context.rsp,this.context);
  });}
  probe(0x1c336d,(sp)=>{active.frame=sp;active.deciding=true;
    if(active.roundBt){roundEmit('btran:applied',sp);active.roundBt=false;}
    emit('factorctl:select',{state:ctlState(),frame:frameState(sp)});});
  probe(0x1c3465,(sp)=>{
    emit('factorctl:call',{decision:!!active.deciding,state:ctlState(),frame:frameState(sp)});active.deciding=false;
    active.savedAlpha=active.s.add(0x210).readPointer();
    active.savedSpike=active.s.add(0x470).readPointer().add(0x1c8).readPointer();
  });
  probe(0x1c3815,(sp,ctx)=>{emit('factorctl:restored',{ret:ctx.rbx.toInt32(),state:ctlState(),frame:frameState(sp),
    alpha_restored:active.s.add(0x210).readPointer().equals(active.savedAlpha),
    spike_restored:active.s.add(0x470).readPointer().add(0x1c8).readPointer().equals(active.savedSpike)});});
  probe(0x1c3aed,(sp)=>{rcFinish(sp,'normal');emit('factorctl:quality_gate',{state:ctlState(),frame:frameState(sp)});});
  probe(0x1c3b6e,(sp)=>{emit('factorctl:clear',{decision:!!active.deciding,state:ctlState(),frame:frameState(sp)});active.deciding=false;});
  Interceptor.attach(b.add(0x115ed0),{
    onEnter(a){this.enabled=active!==null;if(!this.enabled)return;this.inc=a[2];
      emit('factorctl:quality:enter',{enter:a[1].toInt32(),state:ctlState()});},
    onLeave(){if(this.enabled)emit('factorctl:quality:leave',{increment:this.inc.readS32(),state:ctlState()});}
  });
  let recompute = null, recomputeCalls = 0;
  function rcInput(sp) {
    const s=active.s, m=sp.add(0x2068).readS32(), n=s.add(0x68).readS32();
    const f=sp.add(0x2048).readPointer(), input=sp.add(0x2000).readPointer(), output=sp.add(0x2008).readPointer();
    const rhs=sp.add(0x1fd0).readPointer(), basic=sp.add(0x2010).readPointer();
    return {m,header:vec(f,0x18,m,4),rhs:arr(rhs,m,8),lb:vec(sp,0x1f90,n+m),ub:vec(sp,0x1f88,n+m),
      I:hex(sp.add(0x1fc8),8),minus_I:hex(sp.add(0x1f00),8),h:hex(sp.add(0x2070),8),minus_h:hex(sp.add(0x1f08),8),
      attempt:sp.add(0x1fa0).readS32(),model_status:sp.add(0x1f98).readS32(),request:s.add(0x294).readS32(),
      aliases:{rhs_input:rhs.equals(input.add(16).readPointer()),raw_basic:basic.equals(output.add(16).readPointer())},
      frame_matches:{factor:f.equals(s.add(0x470).readPointer()),rhs:rhs.equals(s.add(0x200).readPointer()),
        input:input.equals(s.add(0x228).readPointer()),basic:basic.equals(s.add(0x1e8).readPointer()),
        marks:sp.add(0x2018).readPointer().equals(s.add(0x280).readPointer())}};
  }
  for(const [off,site] of [[0x1c3872,'pivot'],[0x1c3db4,'round']]) probe(off,(sp,ctx)=>{
    if(recompute)throw Error('재계산 중첩');
    recompute={id:++recomputeCalls,site,sp,m:sp.add(0x2068).readS32(),work:ctx.r15};
    emit('recompute:enter',{id:recompute.id,site,input:rcInput(sp)});
  });
  Interceptor.attach(b.add(0x1205d0),{
    onEnter(a){this.rc=recompute;if(!this.rc)return;
      const r=this.rc,sp=r.sp;
      emit('recompute:ftran:enter',{id:r.id,site:r.site,input:{count:a[1].readS32(),values:vec(a[1],16,r.m)},
        args:{factor:a[0].equals(sp.add(0x2048).readPointer()),input:a[1].equals(sp.add(0x2000).readPointer()),
          output:a[2].equals(sp.add(0x2008).readPointer()),flag:a[3].toInt32(),work:a[4].equals(r.work)}});
    },
    onLeave(){if(!this.rc)return;const r=this.rc;
      emit('recompute:ftran:leave',{id:r.id,site:r.site,raw:vec(r.sp.add(0x2008).readPointer(),16,r.m),input_after:vec(r.sp.add(0x2000).readPointer(),16,r.m),rhs_after:vec(r.sp,0x1fd0,r.m)});
    }
  });
  function rcFinish(sp,exit) {
    if(!recompute)return;
    const r=recompute;if(!sp.equals(r.sp))throw Error('재계산 frame 불일치');
    emit('recompute:leave',{id:r.id,site:r.site,exit,
      basic:vec(sp,0x2010,r.m),width:vec(sp,0x1ff0,r.m),marks:vec(sp,0x2018,r.m,1),
      pinf:hex(active.s.add(0x120),8),violations:sp.add(0x1f28).readS32(),retry:active.retry.readS32(),
      request:active.s.add(0x294).readS32(),preserved:{rhs:vec(sp,0x1fd0,r.m),header:vec(sp.add(0x2048).readPointer(),0x18,r.m,4),
        lb:vec(sp,0x1f90,active.s.add(0x68).readS32()+r.m),ub:vec(sp,0x1f88,active.s.add(0x68).readS32()+r.m)},
      aliases:rcInput(sp).aliases});
    recompute=null;
  }
  // 0x1c3aed에는 선행15 probe가 이미 있으므로 동일 주소에 attach를 더하지 않는다.
  probe(0x1c400a,(sp)=>{rcFinish(sp,'normal');roundEmit('recomputed',sp);active.roundFactor=false;active.roundClear=true;});
  probe(0x1c08d5,(sp)=>{rcFinish(sp,'retry');if(active.roundFactor)roundEmit('error',sp);});

  function roundState(sp) {
    const s=active.s,m=s.add(0x64).readS32(),nm=m+s.add(0x68).readS32(),p=sp.add(8),support=sp.add(0x209c);
    const total=p.add(4).readS32();
    const state={m,total,candidates:vec(p,8,total,4),position:p.add(0x20).readS32(),deferred:p.add(0x24).readS32(),
      failures:vec(p,0x10,nm,4),support:support.readS32(),support_marks:active.marksKnown&&support.readS32()>0?vec(p,0x18,m,4):null,
      count_sum:hex(p.add(0x30),8),successes:p.add(0x38).readS32(),batch:[0x1840,0x1844,0x1848].map(o=>p.add(o).readS32())};
    return {...state,marks:active.marksKnown?vec(p,0x18,m,4):null,
      approved:active.passFresh?0:sp.add(0x1ea0).readS32(),mf:hex(sp.add(0x1e98),8),request:s.add(0x294).readS32(),
      status:s.add(0xf0).readS32(),numeric:s.add(0x178).readS32(),failure:s.add(0x184).readS32(),
      attempt:sp.add(0x1fa0).readS32(),numeric_limit:sp.add(0x1fe0).readS32(),retry:active.retry.readS32()};
  }
  function roundEmit(tag,sp,fields={}) {const state=roundState(sp);if(['pass:ready','clear'].includes(tag))delete state.approved;emit('roundctl:'+tag,{state,selected:selected(),...fields});}
  probe(0x1c1ff8,(sp)=>{active.roundSp=sp;active.passStart=true;active.passFresh=true;active.roundFactor=false;
    // approved is reset after the first supplier returns. At this entry its old stack slot is not an input.
    const st=roundState(sp);delete st.approved;emit('roundctl:pass:enter',{state:st});});
  for(const off of [0x1c3bdf,0x1c4e2a])probe(off,(sp,ctx)=>roundEmit('exhausted',sp,{approved_live:ctx.rbp.toInt32()}));
  probe(0x1c3c0c,(sp)=>{
    if(active.roundBt){roundEmit('dense:compacted',sp);active.roundBt=false;}
    roundEmit('restart',sp);
  });
  probe(0x1c3c65,(sp)=>roundEmit('support',sp));
  probe(0x1c3d26,(sp)=>{active.roundFactor=true;roundEmit('factor',sp);});
  probe(0x1c3d65,(sp,ctx)=>roundEmit('factor:return',sp,{ret:ctx.rbp.toInt32()}));
  probe(0x1c4021,(sp,ctx)=>{if(active.roundBt){roundEmit('dense:compacted',sp);active.roundBt=false;}
    roundEmit('end',sp,{remaining:ctx.rdx.toInt32()});});
  probe(0x1c32b5,(sp)=>{
    if(sp.add(0x209c).readS32()===0)return;
    active.roundBt=true;active.marksKnown=true;roundEmit('btran:prepare',sp);
  });
  probe(0x1c45c7,(sp)=>roundEmit('dense',sp));
  Interceptor.attach(b.add(0x131710),{
    onEnter(a){this.enabled=active!==null&&!!active.roundBt;if(!this.enabled)return;
      const sp=active.roundSp,s=active.s,m=s.add(0x64).readS32();this.out=a[2];this.m=m;
      emit('roundctl:btran:enter',{input:sparse(a[1],m),args:{factor:a[0].equals(sp.add(0x2048).readPointer()),
        input:a[1].equals(sp.add(0x1ff8).readPointer()),output:a[2].equals(sp.add(0x2000).readPointer()),
        work:a[3].equals(sp.add(0x2100).readPointer())},selected:selected()});
    },
    onLeave(){if(this.enabled)emit('roundctl:btran:leave',{raw:sparse(this.out,this.m)});}
  });

  // ---- M3-04c-21a: 2207de(F,in,out,save,k5,p6,spike,mark,vals,p10,work) ----
  const ints=(p,n)=>n===0?[]:arr(p,n,4), dbls=(p,n)=>n===0?[]:arr(p,n,8);
  const longs=(p,n)=>{if(n<0||n>100000)throw Error('long 길이 오류');return Array.from({length:n},(_,i)=>p.add(i*8).readS64().toNumber());};
  function nonzero(p,n,size){const r=[];for(let i=0;i<n;i++){const v=size===4?p.add(i*4).readS32():hex(p.add(i*8),8);if(size===4?v!==0:v!=='0000000000000000')r.push([i,v]);}return r;}
  function factorState(F) {
    const m=F.add(8).readS32(), P=o=>F.add(o).readPointer(), I=o=>F.add(o).readS32(), L64=o=>F.add(o).readS64().toNumber();
    const lb=longs(P(0x58),m+1), l0=lb[0], l1=lb[m];
    const ucol=[], urow=[];
    const cs=longs(P(0x88),m), ce=longs(P(0x90),m), cb=longs(P(0x98),m), cv=P(0xa0), cr=P(0xa8);
    for(let j=0;j<m;j++)ucol.push({rows:ints(cr.add(4*cs[j]),ce[j]-cs[j]),vals:dbls(cv.add(8*cs[j]),ce[j]-cs[j])});
    const rs=longs(P(0xb0),m), re=longs(P(0xb8),m), rv=P(0xc8), rc=P(0xd0);
    for(let i=0;i<m;i++)urow.push({cols:ints(rc.add(4*rs[i]),re[i]-rs[i]),vals:dbls(rv.add(8*rs[i]),re[i]-rs[i])});
    const ne=I(0x180), rec=P(0x198), p148=P(0x148), p150=P(0x150), etas=[];
    for(let q=0;q<ne;q++){const e=rec.add(0x18*q), r=e.readS32(), len=e.add(4).readS32(), ip=e.add(8).readPointer(), vp=e.add(0x10).readPointer();
      etas.push({r,len,idx_off:ip.sub(p148).toInt32()/4,val_off:vp.sub(p150).toInt32()/8,idx_mod16:ip.and(0xf).toInt32(),val_mod16:vp.and(0xf).toInt32(),idx:ints(ip,len),val:dbls(vp,len)});}
    const x128=I(0x128);
    return {m,zero:hex(F.add(0x10),8),header:ints(P(0x18),m),perm:{x20:ints(P(0x20),m),x28:ints(P(0x28),m),x30:ints(P(0x30),m),x38:ints(P(0x38),m),x40:ints(P(0x40),m),x48:ints(P(0x48),m)},
      diag:dbls(P(0x50),m),L:{beg:lb,rows:ints(P(0x68).add(4*l0),l1-l0),vals:dbls(P(0x60).add(8*l0),l1-l0),levels:x128,
        lev:x128>=0?ints(P(0x118),m):null,lst:x128>=0?ints(P(0x120),x128+2):null},
      U:{col_start:cs,col_end:ce,col_bound:cb,col:ucol,row_start:rs,row_end:re,row:urow},
      scalars:{xd8:I(0xd8),xdc:I(0xdc),xe4:I(0xe4),xe8:I(0xe8),xec:I(0xec),x100:I(0x100),x104:I(0x104),x108:hex(F.add(0x108),8),x110:hex(F.add(0x110),8),
        x128:x128,x180:ne,x184:I(0x184),x188:I(0x188),x18c:I(0x18c),x158:L64(0x158),x160:L64(0x160),x168:L64(0x168),x1f8:hex(F.add(0x1f8),8),
        pool_mod16:[p148.and(0xf).toInt32(),p150.and(0xf).toInt32()]},
      prefix:{list:ints(P(0xf0),I(0xec)),flag:ints(P(0xf8),m)},eta_oldest:ints(P(0x1a0),m),etas,
      spike:spikeState(P(0x1c8),m),scratch:{x1b0:nonzero(P(0x1b0),m,4),x1b8:nonzero(P(0x1b8),m,8)}};
  }
  let ft=null, ftCalls=0;
  const FT_SITES={0x120730:'supplier',0x120673:'recompute'};
  function ftVec(p,m){const c=p.readS32(),n=c<0?m:c;return {count:c,idx:c<0?null:ints(p.add(8).readPointer(),n),val:dbls(p.add(16).readPointer(),n)};}
  function ftAlias(p){const i=p.add(8).readPointer(),v=p.add(16).readPointer(),t=x=>x.equals(ft.inIdx)?'in':x.equals(ft.outIdx)?'out':'other',u=x=>x.equals(ft.inVal)?'in':x.equals(ft.outVal)?'out':'other';return {idx:t(i),val:u(v)};}
  Interceptor.attach(b.add(0x120740),{
    onEnter(a){
      const site=FT_SITES[this.returnAddress.sub(b).toUInt32()];
      this.enabled=!!site&&(site==='supplier'?supplier!==null:recompute!==null);if(!this.enabled)return;
      if(ft)throw Error('FTRAN 중첩');
      const F=a[0], m=F.add(8).readS32(), spike=a[6].isNull()?null:a[6];
      ft={id:++ftCalls,site,F,m,in:a[1],out:a[2],spike,inIdx:a[1].add(8).readPointer(),inVal:a[1].add(16).readPointer(),outIdx:a[2].add(8).readPointer(),outVal:a[2].add(16).readPointer(),work:a[10]};
      const ctx=site==='supplier'?{supplier_id:supplier.id,factor_s470:F.equals(active.s.add(0x470).readPointer()),input_s220:a[1].equals(active.s.add(0x220).readPointer()),output_s210:a[2].equals(active.s.add(0x210).readPointer())}
        :{recompute_id:recompute.id,recompute_site:recompute.site,factor_frame:F.equals(recompute.sp.add(0x2048).readPointer()),input_frame:a[1].equals(recompute.sp.add(0x2000).readPointer()),output_frame:a[2].equals(recompute.sp.add(0x2008).readPointer())};
      const clear=site==='supplier'&&ftClearList&&ftClearList.indices?{before:ftClearList,values_now:ftClearList.indices.map(r=>hex(F.add(0x1c8).readPointer().add(16).readPointer().add(r*8),8))}:null;
      ftClearList=null;
      emit('ftran:enter',{id:ft.id,site,ctx,args:{save:a[3].toInt32(),k5:a[4].toInt32(),p6_null:a[5].isNull(),spike_null:spike===null,spike_f1c8:spike!==null&&spike.equals(F.add(0x1c8).readPointer()),
        vals_null:a[7].isNull(),mark_null:a[8].isNull(),p10_null:a[9].isNull(),work_null:a[10].isNull()},
        input:ftVec(a[1],m),factor:factorState(F),clear,work:a[10].isNull()?null:hex(a[10],8)});
    },
    onLeave(){
      if(!this.enabled)return;
      const F=ft.F,m=ft.m;
      emit('ftran:leave',{id:ft.id,site:ft.site,output:ftVec(ft.out,m),input_after:ftVec(ft.in,m),alias:{in:ftAlias(ft.in),out:ftAlias(ft.out)},
        factor:factorState(F),work:ft.work.isNull()?null:hex(ft.work,8)});
      ft=null;
    }
  });
  for(const [name,off,ret] of [['214344',0x12bd30,0x1208bb],['21b9de',0x12a990,0x120927],['21574d',0x123690,0x1219ae],['2126cd',0x121ef0,0x121ca6],['21686f',0x1223e0,0x121cd1]]) Interceptor.attach(b.add(off),{
    onEnter(){this.enabled=ft!==null&&this.returnAddress.sub(b).toUInt32()===ret;if(!this.enabled)return;
      emit('ftran:kernel',{id:ft.id,kernel:name,phase:'enter',input:ftVec(ft.in,ft.m),alias:{in:ftAlias(ft.in),out:ftAlias(ft.out)},spike:spikeState(ft.F.add(0x1c8).readPointer(),ft.m)});},
    onLeave(){if(!this.enabled)return;
      emit('ftran:kernel',{id:ft.id,kernel:name,phase:'leave',input:ftVec(ft.in,ft.m),output:ftVec(ft.out,ft.m),alias:{in:ftAlias(ft.in),out:ftAlias(ft.out)},spike:spikeState(ft.F.add(0x1c8).readPointer(),ft.m)});}
  });
}});
