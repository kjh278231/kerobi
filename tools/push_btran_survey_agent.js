// M3-04c-21c 조사: push 안 지원 BTRAN(09e376→221173)의 경로 요약만 센다. 읽기 전용, 판정 자료 아님.
Process.attachModuleObserver({onAdded(mod) {
  if (mod.name !== 'libgurobi130.so') return;
  const b = mod.base;
  let push = 0, bt = null;
  Interceptor.attach(b.add(0x1c0070), {onEnter() {push++;}, onLeave() {push--;}});
  Interceptor.attach(b.add(0x12de60), {
    onEnter(a) {
      this.on = push > 0 && this.returnAddress.sub(b).toUInt32() === 0x131730; if (!this.on) return;
      const F = a[0], m = F.add(8).readS32(), cnt = a[1].readS32();
      const r = cnt > 0 ? a[1].add(8).readPointer().readS32() : null;
      const urow = r === null ? null : F.add(0xb8).readPointer().add(8*r).readS64().toNumber() - F.add(0xb0).readPointer().add(8*r).readS64().toNumber();
      bt = {m, cnt, r, urow, eta: F.add(0x180).readS32(), x104: F.add(0x104).readS32(), x140: F.add(0x140).readS32(),
        s: Math.trunc(F.add(0x108).readDouble() + F.add(0x110).readDouble()), cache_null: a[3].isNull(), kin: null, kout: null};
      this.F = F; this.out = a[2];
    },
    onLeave() {
      if (!this.on) return;
      const c = this.F.add(0x1d0).readPointer();
      bt.out = this.out.readS32(); bt.cache = c.isNull() ? null : c.readS32(); bt.x1d8 = this.F.add(0x1d8).readS32();
      send({event: 'bt', ...bt}); bt = null;
    }
  });
  Interceptor.attach(b.add(0x130b70), {
    onEnter(a) {this.on = bt !== null && this.returnAddress.sub(b).toUInt32() === 0x12e58b; if (this.on) {bt.kin = a[3].readS32(); this.o = a[4];}},
    onLeave() {if (this.on) bt.kout = this.o.readS32();}
  });
}});
