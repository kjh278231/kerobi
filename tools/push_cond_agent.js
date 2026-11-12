// M3-04c-1c: M3-04c-1b agent(push_init_agent.js) + 조건수 116fd7 내부 경계(진입·09cf9b·9e288·반환) 읽기 전용 관측.
// 1b 사건은 그대로 두고 cond:* 사건만 더한다(비간섭 투영 비교). 아래 1b 설명은 원문 그대로.
// M3-04c-1b: primal push 초기화(134a87 C 150–1142)의 중간 경계 읽기 전용 관측과 원본 명령 합성용 입력 편집.
// push:enter·첫 supply:enter 페이로드는 21h agent(push_block_agent.js)와 같은 필드로 만든다(비간섭 투영 비교).
// CONFIG.edits: [{at:'enter'|'ftran', field:'s120'|'s390'|'s3c8'|'vstat'|'x'|'slack', index?, hex?|int?}] 첫 push 호출에서만 적용.
// CONFIG.survey: true면 배열 없이 분기 요약만 보낸다(자연 탐색).
const CONFIG = __CONFIG__;
Process.attachModuleObserver({onAdded(mod) {
  if (mod.name !== 'libgurobi130.so') return;
  const b = mod.base, survey = !!CONFIG.survey, edits = CONFIG.edits || [];
  let active = null, calls = 0;
  const hex = (p, size) => Array.from(new Uint8Array(p.readByteArray(size))).map(x => x.toString(16).padStart(2, '0')).join('');
  function arr(p, n, size) {
    if (n < 0 || n > 1000000) throw Error('배열 길이 오류: ' + n);
    if (p.isNull()) throw Error('필수 배열 null');
    return Array.from({length:n}, (_, i) => size === 4 ? p.add(i*size).readS32() : size === 1 ? hex(p.add(i), 1) : hex(p.add(i*size), size));
  }
  const vec = (s, off, n, size=8) => arr(s.add(off).readPointer(), n, size);
  const S = () => active.s, M = () => active.s.add(0x64).readS32(), N = () => active.s.add(0x68).readS32();
  function contextState(p, support, m, nm) {
    const total=p.add(4).readS32(), sp=support.readS32();
    return {m:p.readS32(),total,candidates:vec(p,8,total,4),position:p.add(0x20).readS32(),
      deferred:p.add(0x24).readS32(),failures:vec(p,0x10,nm,4),support:sp,
      support_marks:sp>0?vec(p,0x18,m,4):null,count_sum:hex(p.add(0x30),8),successes:p.add(0x38).readS32(),
      batch:[0x1840,0x1844,0x1848].map(o=>p.add(o).readS32())};
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
  // S+0x4b8 → +0x100 → +0x2c54 (061bfe가 읽는 환경 flag), 갱신 한도 S+0x3c8·F+0x188, S+0x390, S+0x120, S+0x178.
  function extra() {
    const s=S(), env=s.add(0x4b8).readPointer();
    let flag=null;
    // 061bfe C 13: int 비교(< 1).
    if(!env.isNull()){const e2=env.add(0x100).readPointer(); if(!e2.isNull()) flag=e2.add(0x2c54).readS32();}
    return {s120:hex(s.add(0x120),8),s390:hex(s.add(0x390),8),s3c8:s.add(0x3c8).readS32(),f188:s.add(0x470).readPointer().add(0x188).readS32(),
      env_2c54:flag,numeric:s.add(0x178).readS32(),status:s.add(0xf0).readS32(),request:s.add(0x294).readS32()};
  }
  function emit(event, fields={}) {send({event,call:active.call,...fields});}
  function summary(){return active.sum;}
  function applyEdits(at) {
    if(active.call!==1) return;
    for(const e of edits) {
      if(e.at!==at) continue;
      const s=S(); let p, size=8;
      if(e.field==='s120') p=s.add(0x120);
      else if(e.field==='s390') p=s.add(0x390);
      else if(e.field==='s3c8') {p=s.add(0x3c8); size=4;}
      else if(e.field==='vstat') {p=s.add(0x168).readPointer().add(e.index*4); size=4;}
      else if(e.field==='x') p=active.x.add(e.index*8);
      else if(e.field==='slack') p=active.slack.add(e.index*8);
      else throw Error('알 수 없는 편집 '+e.field);
      const before=size===4?p.readS32():hex(p,8);
      if(size===4) p.writeS32(e.int); else p.writeByteArray(e.hex.match(/../g).map(h=>parseInt(h,16)));
      const after=size===4?p.readS32():hex(p,8);
      emit('edit',{at,field:e.field,index:e.index===undefined?null:e.index,before,after});
    }
  }
  Interceptor.attach(b.add(0x1c0070),{
    onEnter(a) {
      if(active) throw Error('push 중첩');
      active={call:++calls,s:a[0],x:a[3],slack:a[4],condition:a[6],retry:a[7],remaining:a[8],init:true,ftran:0,
        first:a[5].toInt32(),sum:{}};
      const s=a[0], n=s.add(0x68).readS32(), m=s.add(0x64).readS32();
      if(survey) {
        const vs=vec(s,0x168,n+m,4), hist={};
        vs.forEach((v,k)=>{if(v<0){const key=(k<n?'c':'s')+v;hist[key]=(hist[key]||0)+1;}});
        Object.assign(active.sum,{n,m,first:a[5].toInt32(),attempt:a[2].toInt32(),model_status:a[1].toInt32(),vstat:hist,enter:extra(),
          precision:s.add(0x28).readS32()});
      } else {
        const starts=vec(s,0x70,n,8), lengths=vec(s,0x78,n,4);
        let nnz=0;for(let j=0;j<n;j++) nnz=Math.max(nnz,Number(s.add(0x70).readPointer().add(j*8).readS64())+lengths[j]);
        emit('push:enter',{first:a[5].toInt32(),model_status:a[1].toInt32(),attempt:a[2].toInt32(),alias_contiguous:a[4].equals(a[3].add(n*8)),
          model:{starts,lengths,rows:vec(s,0x80,nnz,4),values:vec(s,0x88,nnz),cost:vec(s,0x98,n+m),rhs:vec(s,0xa0,m),lb:vec(s,0xa8,n+m),ub:vec(s,0xb0,n+m)},state:snap()});
        emit('init:enter',{extra:extra(),residual_scratch:vec(s,0x1f8,m),condition_in:hex(a[6],8)});
      }
      applyEdits('enter');
    },
    onLeave(r) {
      const fields={ret:r.toInt32(),condition:hex(active.condition,8),retry:active.retry.readS32(),remaining:active.remaining.readS32(),init:active.init,extra:extra()};
      if(survey) {Object.assign(active.sum,{leave:fields}); send({event:'survey',call:active.call,...active.sum});}
      else emit('push:leave',fields);
      active=null;
    }
  });
  Interceptor.attach(b.add(0x2937b0),{onEnter(){send({event:'unsupported:x87'});}});
  function onInit(){return active!==null && active.init;}
  // τ_in: 0x1c0239 movsd %xmm0,0x1ee0(%rsp) 직후.
  Interceptor.attach(b.add(0x1c0242),function(){
    if(!onInit())return;
    if(!this.context.r14.equals(S()))throw Error('workspace ABI 불일치');
    // 0x2070(τ)은 0x1c0284에서야 쓰이므로 여기서 읽지 않는다.
    const sp=this.context.rsp, v={tau_in:hex(sp.add(0x1ee0),8)};
    if(survey) active.sum.tau=v; else emit('init:tau',v);
  });
  // 초기 재분해(요청 10): caller 0x1c07c6·0x1c07ea.
  Interceptor.attach(b.add(0xf15c0),{
    onEnter(){
      const c=this.returnAddress.sub(b).toInt32();
      this.on=onInit()&&(c===0x1c07c6||c===0x1c07ea);if(!this.on)return;
      this.caller=c;this.request=S().add(0x294).readS32();
      // 1c: 초기 재분해 입력(Rust SerialStart::new) — S+0x2fc·0x2f8·0x46c, 036ade 진입 블록 상태
      if(!survey){const s=S();this.pre={x2fc:s.add(0x2fc).readS32(),x2f8:s.add(0x2f8).readS32(),x46c:s.add(0x46c).readS32()};active.blk=[];}
    },
    onLeave(r){
      if(!this.on)return;
      if(!survey){const F=S().add(0x470).readPointer();
        emit('cond:lu',{caller:this.caller.toString(16),pre:this.pre,block:active.blk,zero:hex(F.add(0x10),8),x158:F.add(0x158).readS64().toNumber(),
          pool_mod16:[F.add(0x148).readPointer().and(0xf).toInt32(),F.add(0x150).readPointer().and(0xf).toInt32()],
          x188:F.add(0x188).readS32(),x1f8:hex(F.add(0x1f8),8)});active.blk=null;}
      const v={caller:this.caller.toString(16),request_in:this.request,ret:r.toInt32(),extra:extra()};
      if(survey) (active.sum.lu=active.sum.lu||[]).push(v); else emit('init:lu',{...v,header:vec(S().add(0x470).readPointer(),0x18,M(),4),vstat:vec(S(),0x168,N()+M(),4)});
    }
  });
  // 116fd7 안의 벡터 구조(+0 count, +8 idx*, +0x10 val*): 버퍼 이름은 진입 때 x(S+0x220)·y(S+0x228)의 포인터.
  function bufName(p){const c=active.cond;for(const k of ['X','Y','Xi','Yi']) if(p.equals(c.ptr[k])) return k;return 'other:'+p.and(0xf).toInt32();}
  function header(v){
    const count=v.readS32(), m=M(), ip=v.add(8).readPointer(), vp=v.add(16).readPointer();
    return {count,idx:count>=0?arr(ip,count,4):null,val:arr(vp,count>=0?count:m,8),idx_buf:bufName(ip),val_buf:bufName(vp)};
  }
  function onCond(){return active!==null && active.init && active.cond!==null && active.cond!==undefined;}
  // 116fd7 진입: 12f282가 tail call(jmp)로 넘긴다. 0x1c17b3 call 자체에 훅이 있어 return address는 Frida
  // trampoline을 가리키므로(탐색 확인) 0x1c17b3 직전 훅~0x1c17b8 직후 훅 사이(active.inCond)인지로 식별한다.
  Interceptor.attach(b.add(0x1a92c0),{
    onEnter(a){
      this.on=onInit()&&active.inCond===true;if(!this.on)return;
      if(survey) throw Error('survey 미지원');
      const s=a[0].add(0xe0).readPointer();
      if(!s.equals(S())) throw Error('116fd7 workspace ABI 불일치');
      const f=s.add(0x470).readPointer(), x=s.add(0x220).readPointer(), y=s.add(0x228).readPointer(), m=M();
      active.cond={k:0,ptr:{X:x.add(16).readPointer(),Y:y.add(16).readPointer(),Xi:x.add(8).readPointer(),Yi:y.add(8).readPointer()}};
      this.out=a[1];
      const P=active.cond.ptr;
      emit('cond:enter',{m,n:N(),f0c:f.add(0xc).readS32(),precision:s.add(0x28).readS32(),header:vec(f,0x18,m,4),
        a_val_mod16:s.add(0x88).readPointer().and(0xf).toInt32(),mod16:Object.fromEntries(Object.entries(P).map(([k,p])=>[k,p.and(0xf).toInt32()])),
        distinct:new Set(Object.values(P).map(p=>p.toString())).size,out_is_param:a[1].equals(active.condPtr),work_null:a[2].isNull(),condition_in:hex(a[1],8)});
    },
    onLeave(r){
      if(!this.on)return;
      emit('cond:leave',{ret:r.toInt32(),condition:hex(this.out,8)});
      active.cond=null;
    }
  });
  Interceptor.attach(b.add(0x285d00),{onEnter(){send({event:'unsupported:23e604'});}});
  // 036ade 진입(05b257 caller 0xf34f2): 초기 재분해 중이면 블록 상태 S+0x2c0/0x4e0/0x4e4.
  Interceptor.attach(b.add(0xd4190),{onEnter(a){
    if(active===null||!Array.isArray(active.blk)||this.returnAddress.sub(b).toInt32()!==0xf34f2)return;
    if(!a[0].equals(S()))throw Error('036ade S 불일치');
    active.blk.push({x2c0:a[0].add(0x2c0).readS32(),x4e0:a[0].add(0x4e0).readS32(),x4e4:a[0].add(0x4e4).readS32()});
  }});
  // 9e288 BTRAN: caller 0x1a998a(116fd7)만.
  Interceptor.attach(b.add(0x12de10),{
    onEnter(a){
      this.on=onCond()&&this.returnAddress.sub(b).toInt32()===0x1a998a;if(!this.on)return;
      const s=S();
      if(!a[0].equals(s.add(0x470).readPointer())||!a[1].equals(s.add(0x228).readPointer())||!a[2].equals(s.add(0x220).readPointer())) throw Error('9e288 인자 불일치');
      this.x=a[2];this.y=a[1];
      emit('cond:btran:enter',{k:active.cond.k,input:header(a[1]),output:header0(a[2])});
    },
    onLeave(){
      if(!this.on)return;
      emit('cond:btran:leave',{k:active.cond.k,output:header(this.x),input:header0(this.y)});
      active.cond.k++;
    }
  });
  // 값 없이 헤더 이름만(입력 쪽 사후 상태·출력 쪽 사전 상태).
  function header0(v){return {count:v.readS32(),idx_buf:bufName(v.add(8).readPointer()),val_buf:bufName(v.add(16).readPointer())};}
  // 09cf9b: caller 0x1c11d1(dense rhs FTRAN)과 0x1a966f(116fd7).
  Interceptor.attach(b.add(0x1205d0),{
    onEnter(a){
      const ra=this.returnAddress.sub(b).toInt32();
      if(ra===0x1a966f&&onCond()){
        const s=S();
        if(!a[0].equals(s.add(0x470).readPointer())||!a[1].equals(s.add(0x220).readPointer())||!a[2].equals(s.add(0x228).readPointer())||a[3].toInt32()!==0) throw Error('116fd7 09cf9b 인자 불일치');
        this.cond=true;this.x=a[1];this.y=a[2];
        emit('cond:ftran:enter',{k:active.cond.k,input:header(a[1]),output:header0(a[2])});
        return;
      }
      this.cond=false;
      this.on=onInit()&&ra===0x1c11d1;if(!this.on)return;
      const s=S(), m=M();
      active.ftran++;
      if(active.ftran===1) applyEdits('ftran');
      this.out=a[2];this.m=m;
      const args={factor:a[0].equals(s.add(0x470).readPointer()),input:a[1].equals(s.add(0x228).readPointer()),output:a[2].equals(s.add(0x220).readPointer()),flag:a[3].toInt32()};
      if(survey) {(active.sum.ftran=active.sum.ftran||[]).push({count:a[1].readS32(),args});return;}
      emit('init:ftran:enter',{n:active.ftran,count:a[1].readS32(),values:vec(a[1],16,m),args,extra:extra()});
    },
    onLeave(){
      if(this.cond){emit('cond:ftran:leave',{k:active.cond.k,output:header(this.y),input:header0(this.x)});return;}
      if(!this.on||survey)return;
      emit('init:ftran:leave',{n:active.ftran,count:this.out.readS32(),values:vec(this.out,16,this.m)});
    }
  });
  // 089d8f 잔차: caller 0x1c1287·0x1c14a2.
  Interceptor.attach(b.add(0x115d60),{
    onEnter(a){
      const c=this.returnAddress.sub(b).toInt32();
      this.on=onInit()&&(c===0x1c1287||c===0x1c14a2);if(!this.on)return;
      const s=S(), m=M();this.outp=a[2];this.m=m;this.caller=c;
      const args={workspace:a[0].equals(s),xb_is_s220:a[1].equals(s.add(0x220).readPointer().add(16).readPointer()),out_is_s1f8:a[2].equals(s.add(0x1f8).readPointer())};
      if(survey) {(active.sum.residual=active.sum.residual||[]).push({caller:c.toString(16),args});return;}
      emit('init:residual:enter',{caller:c.toString(16),args,x:arr(a[1],m,8),out:arr(a[2],m,8)});
    },
    onLeave(){
      if(!this.on||survey)return;
      emit('init:residual:leave',{caller:this.caller.toString(16),out:arr(this.outp,this.m,8)});
    }
  });
  // Markowitz 강화: 0x1c151c call 12f5d1 직전(새 값 0x1e70, 잔차 0x1eb8), 0x1c1521 직후.
  Interceptor.attach(b.add(0x1c151c),function(){
    if(!onInit())return;
    const sp=this.context.rsp, v={markowitz_new:hex(sp.add(0x1e70),8),residual:hex(sp.add(0x1eb8),8),extra:extra()};
    if(survey) (active.sum.strengthen=active.sum.strengthen||[]).push(v); else emit('init:strengthen:enter',v);
  });
  Interceptor.attach(b.add(0x1c1521),function(){
    if(!onInit())return;
    const v={extra:extra()};
    if(survey) (active.sum.strengthen_leave=active.sum.strengthen_leave||[]).push(v); else emit('init:strengthen:leave',v);
  });
  // 품질 gate: 0x1c17b3 call 12f282 직전·0x1c17b8 직후.
  Interceptor.attach(b.add(0x1c17b3),function(){
    if(!onInit())return;
    const sp=this.context.rsp, cp=sp.add(0x20e0).readPointer();
    active.condPtr=cp;active.inCond=true;
    const v={pinf:hex(sp,8),violations:sp.add(0x1f28).readS32(),condition_before:hex(cp,8),cond_is_param:cp.equals(active.condition),extra:extra()};
    if(survey) active.sum.cond_enter=v; else emit('init:condition:enter',v);
  });
  Interceptor.attach(b.add(0x1c17b8),function(){
    if(!onInit())return;
    active.inCond=false;
    const v={condition:hex(active.condPtr,8)};
    if(survey) active.sum.cond_leave=v; else emit('init:condition:leave',v);
  });
  // 기준 조정 판정 직전(gate 통과 뒤).
  Interceptor.attach(b.add(0x1c184d),function(){
    if(!onInit())return;
    if(!this.context.r14.equals(S()))throw Error('workspace ABI 불일치');
    const sp=this.context.rsp;
    const v={adjusted:sp.add(0x1ec0).readU8(),candidates:sp.add(0x1fb0).readS32(),count01:sp.add(0x1e88).readS32(),
      count0001:sp.add(0x1e80).readS32(),count001:sp.add(0x1e78).readS32(),tau_in:hex(sp.add(0x1ee0),8),h:hex(sp.add(0x2070),8),
      // (%rsp)의 PInf는 0x1c17a7(first≠0 경로)에서만 쓰인다. first=0이면 미초기화 스택이므로 읽지 않는다.
      pinf_slot:active.first?hex(sp,8):null,extra:extra()};
    if(survey) (active.sum.gate=active.sum.gate||[]).push(v); else emit('init:adjust-gate',v);
  });
  // 첫 공급자 진입 = 초기화 종료 경계. 21h supply:enter와 같은 필드.
  Interceptor.attach(b.add(0x1c5310),{
    onEnter(a) {
      if(!onInit())return;
      active.init=false;
      const p=a[0], total=p.add(4).readS32(), s=S(), nm=M()+N();
      if(survey){active.sum.supply={total,support:a[3].readS32(),extra:extra(),pinf:hex(s.add(0x120),8)};return;}
      emit('supply:enter',{candidates:vec(p,8,total,4),position:p.add(0x20).readS32(),failures:vec(p,0x10,nm,4),support:a[3].readS32(),support_marks:a[3].readS32()>0?vec(p,0x18,M(),4):null,batch:p.add(0x1840).readS32(),state:snap(true),supplier:{id:1,context:contextState(p,a[3],M(),nm),drop:hex(s.add(0x3a8),8)}});
    }
  });
}});
