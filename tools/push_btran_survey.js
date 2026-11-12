// M3-04c-21c 조사: 여러 모델을 한 Gurobi 프로세스에서 순서대로 풀며 지원 BTRAN 경로 요약을 모은다.
const fs = require('fs'), path = require('path'), crypto = require('crypto');
const frida = require('/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida');
const [output, ...models] = process.argv.slice(2);
if (!output || !models.length) throw Error('사용법: OUTPUT MODEL...');
if (fs.existsSync(output)) throw Error('기존 출력 덮어쓰기 거부');
const root = path.resolve(__dirname, '../..'), py = path.join(root, '.venv/bin/python');
const binary = path.join(root, '.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so');
const hash = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
if (hash(binary) !== '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c') throw Error('원본 바이너리 해시 불일치');
const params = {Threads: 1, Seed: 0, OutputFlag: 0, Presolve: 0, Method: 2, Crossover: 3, LPWarmStart: 2};
const code = `import time\nwhile True:\n try:\n  import gurobipy as gp\n  break\n except RuntimeError as exc:\n  if "Gurobi is already running" not in str(exc): raise\n  time.sleep(0.25)\ne=gp.Env(params=${JSON.stringify(params)})\nfor p in ${JSON.stringify(models.map(m => path.resolve(m)))}:\n print('MODEL '+p,flush=True)\n m=gp.read(p,e)\n m.optimize()\n print('STATUS',m.Status,flush=True)\n m.dispose()\n`;
const agentPath = path.join(__dirname, 'push_btran_survey_agent.js'), agent = fs.readFileSync(agentPath, 'utf8');
(async () => {
  const device = await frida.getLocalDevice(), events = [], errors = []; let out = '', model = null;
  const pid = await device.spawn([py, '-c', code], {stdio: 'pipe'}), session = await device.attach(pid), script = await session.createScript(agent);
  script.message.connect(msg => {if (msg.type === 'send') events.push({model, ...msg.payload}); else errors.push(msg.stack || JSON.stringify(msg));});
  device.output.connect((p, fd, data) => {if (p !== pid) return; const s = data.toString(); out += s; for (const l of s.split('\n')) if (l.startsWith('MODEL ')) model = path.basename(l.slice(6));});
  session.detached.connect(reason => {
    fs.writeFileSync(output, JSON.stringify({pid, agent_sha256: hash(agentPath), tool_sha256: hash(__filename), params, models: models.map(m => [m, hash(m)]), output: out, reason, errors, events}, null, 1) + '\n');
    console.log('사건', events.length, errors.join(' ')); process.exit(errors.length ? 1 : 0);
  });
  await script.load(); await device.resume(pid);
})().catch(e => {console.error(e); process.exit(1);});
