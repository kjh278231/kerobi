# M2-06h-6 SB 비전파 경로(VarBranch=3) 오라클·Rust 이식 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 기준 main `3f19629`, 미커밋 선행 변경 없음. 작업자는 이 세션(Claude Code, "다음 스프린트 진행해 너가 작업자야" 지시). branch `work/m2-06h-6`, worktree `../rsolver-m2-06h-6`. 점유 기록의 branch는 main으로 찍혔다(06h-4와 같은 현상, 실제는 위 branch).

## 1. 범위와 주장

- 작업 종류: 오라클 구축 + 구현(L1). 06h-3 재생 하네스를 확장한다.
- 완료할 동작: `695bd7` 후보 루프의 **비전파 경로**(`local_90 = 0`). 후보 방향마다 전파 대신 후보 열 자신의 bound를 `d8d5f`로 직접 요청하고(up `'>'` floor+1, down `'<'` floor), flush·풀이·결과 소비·사전 채움·reset·restore 뒤 같은 쪽 bound를 **무조건** 다시 요청한다. 복구 값은 후보 시작 때 LP 내부 bound(`+0xe8` 자료의 `0x390`·`0x398`)를 열 배율(`0x3e0`)로 곱하고 무한 문턱(env `+0x2458`) 밖이면 ∓1e100으로 둔 값이다. 6·개수 work는 더하지 않는다.
- 입력 전제: Threads=1, 정수(B/I) 후보, stride 1, 뒤집힌 열 아님(`+0x3c ≠ 1` 또는 열 flag 0). 나머지 루프 제어(게이트·사전 채움·점수·stall·타이머 반응)는 06h-3·06h-5와 공유한다.
- 비전파 진입 조건: C:343 `local_68(Node) ≠ 0 && Node+0x18 ≠ 0`일 때만 전파 배열을 잡는다. 이번 캡처는 Node와 `Node+0x18`의 null 여부를 커널 사건에 직접 기록하고 하네스는 그 값으로 경로를 고른다(사건 누락으로 추정하지 않는다). Node = NULL 호출자는 `62a05d` C:785(asm 0x538810 `xor %esi,%esi` → 0x53882b `691b56`)다. 그 블록의 진입 조건(VarBranch == 3 이외)이 배타적인지는 이번에도 조사 범위 밖이다.
- 제외(미검증으로 남김): 뒤집힌 열 복구 값(C:398–405·C:409–411, `-lb·scale`·−1e100; 자연 0회, 하네스가 거부), 연속 열 `'C'` 후보(down에서 LB·UB 둘 다 요청, C:841–850·C:973–990; 자연 0회, 하네스 거부), 비전파 경로의 상태 7·11, 타이머 만료, up ≥ 1e100, stall 조기 종료(이 호출자는 stall 2e9).
- **범위 분할**: 스프린트 표의 "모순·연속 열 탐색"은 이번 번호에서 하지 않고 **M2-06h-7**(조사)로 나눈다. 이유: 비전파 이식과 독립이고, 06h-4에서 33모델 NodeLimit 300으로 0회였으므로 깊은 NodeLimit·클리크·SOS 모델 탐색이 따로 1주 범위다. 06h-5가 가칭 06h-7로 적은 타이머 산술은 **M2-06h-8**로 번호를 옮긴다.
- 선행: M2-06h-3·06h-4·06h-5 자료와 하네스.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- C `../re/decomp/c/00571770_PRIVATE0000000000695bd7.c`.
  - C:343–377: 전파 배열·원래 bound getter는 `Node ≠ 0 && Node+0x18 ≠ 0` 안에서만. 비전파는 getter를 부르지 않는다(사전 단계 = `factorSave, save, input, attr`).
  - C:395–431: 후보 시작. `lb = d[0x390][j]`, 문턱 `inf = env[0x2458]`. `-inf < lb`면 `local_168 = lb·scale[j]`(배율 배열 null이면 lb), 아니면 −1e100. `ub = d[0x398][j]`, `ub < inf`면 `ub·scale[j]`, 아니면 1e100(`local_170`). 뒤집힌 열(`d+0x3c == 1 && d[0x408][j] == 1`)은 −1e100과 `-lb·scale`(제외).
  - C:454–462: up 비전파 요청 `d8d5f(LP, 1, &j, &(floor+1), '>')`. C:583–586: reset·restore 뒤 `d8d5f(LP, 1, &j, &local_168, '>')` 무조건, work 없음.
  - C:841–855: down 비전파 요청 `d8d5f(LP, 1, &j, &floor, '<')`('C'면 앞에 `'>'` floor도). C:974–990: reset·restore 뒤 `d8d5f(LP, 1, &j, &local_170, '<')` 무조건.
  - 그 밖의 흐름(flush·풀이·소비·사전 채움·게이트·점수·타이머)은 [06h-3 §2](mip_sb_loop_rust.md#2-원본과-상태-경계)와 같은 줄을 지난다.
- `d8d5f`(0x1690d0) 훅 `boundwrite`는 06h-3 도구에 이미 있다(값은 요청 값, 배율 적용 전).
- 시작 경계: 저장 객체·부모 ObjVal·해를 얻은 뒤(첫 `poll` 직전). 종료: 커널 반환.
- 입력: 후보 열 순서, 부모 해 x, IntFeasTol, 부모 ObjVal, stall 한도, 커널 진입 때 LP 내부 bound(`bounds0`)·열 배율·무한 문턱, 시작 반복 비용·work. backend 출력: LP 상태·ObjVal/dual·IterCount, 뒤 후보 해 값, 타이머 반환, restore 안 work.

## 3. 비교 계약(첫 Rust 재생 전 고정)

| 필드·사건 | 규칙 | 근거 |
|---|---|---|
| 명령 순서(poll, timer, bound, flush, solve, values, reset, restore, 종료 flush·solve) | 정확 일치 | 원본 훅 순서 |
| 직접 요청(열·방향·값) | 정확·u64 비트 | `boundwrite` 다음이 flush |
| 복구 요청(열·방향·값) | 정확·u64 비트, 무조건 | `boundwrite` 다음이 flush 아님 |
| work | 다음 기록 사건의 원본 work와 비트 일치(Rust는 비전파에서 더하지 않음) | C:583–586, 974–990 |
| 최종 down/up 배열·반복 비용 | 후보별 u64 비트 | 커널 반환 |

- 복구 값의 입력은 그 커널 사건의 `bounds0`·`input.scales`·`input.inf`에서만 만든다. 원본 `boundwrite` 값을 Rust 입력으로 넣지 않는다. 하네스는 원본 자료 단계에서 후보 열의 `bounds0` 값이 커널 안 어느 flush 뒤에도 같은 열 요청 외에는 바뀌지 않는다는 전제를 따로 두지 않고, Rust 재생 비교로 판정한다.
- 오차 0, ±0 보존. NaN·IEEE 무한 입력은 거부.
- 필수 사례(원본 자료에서 센다, 비전파 묶음만): `direct request`, `direct restore`, `direct status 2`, `direct status -1000`, `direct status 3`, `direct values`, `direct down >= 1e100`. 전파 경로의 기존 필수 사례 목록은 그대로다.
- 판별력(기대를 먼저 적음):
  - 변조(원본 자료 단계 또는 Rust 재생에서 거부): `direct flag`(비전파 묶음의 `propagation`을 참으로 → 사전 단계 불일치), `direct inf`(문턱을 1.0으로 → 복구 값 차이), `direct scale`(배율 하나를 2.0으로 → 복구 값 차이), `direct bounds`(후보 열 `bounds0` lb를 다른 값으로 → 복구 값 차이), `no direct`(비전파 묶음 제거 → 필수 사례 0).
  - Rust 변이(원본 재생에서 거부): `direct restore drop`(복구 요청 없음), `direct restore gated`(복구 값이 0이면 요청 생략), `direct work`(복구 뒤 `+6`), `direct side`(복구 요청 방향 반대), `direct scale`(배율 무시), `direct request`(직접 요청 값 +1).
  - `direct clamp`(문턱 밖 값을 그대로)는 원본 재생에서 **생존**할 것으로 미리 적는다. 후보가 모두 이진 열이라 내부 bound가 문턱 밖인 사례가 없다(markshare_4_0 첫 묶음: lb −0, ub 8·16, 배율 0.125·0.0625). 합성 테스트가 거부해야 한다.
  - 생존 변이 목록은 기존 `restore strict`·`score shift`에 `direct clamp`를 더한 셋이어야 한다.

## 4. 사례와 원본 자료

설정: Threads=1, Seed=0, LogToConsole=0, NodeLimit=20, GURO_PAR_SBMAXITERS=10, VarBranch=3. 캡처 도구 `tools/mip_sb_direct.js`(06h-3 `mip_sb_loop.js` 치환 + null 안전 + `node`·`propagation`·`inf` 필드). 모델마다 무훅 대조 1회 + 독립 3회. 비간섭: 같은 도구로 기본 설정 pk1(NodeLimit 300)을 1회 캡처해 새 필드·kind·도구 해시를 뺀 투영이 `bench/oracle/m2_06h3/final/pk1_1.json.gz`와 같아야 한다.

| 사례 | 실제/합성 | 노리는 분기 | 탐색 증거(scratch, 기준 아님) |
|---|---|---|---|
| markshare_4_0 VB3 | 실제 | 직접 요청·복구, 상태 2·−1000, 사전 채움 | 20묶음, 상태 2 120·−1000 6 |
| pk1 VB3 | 실제 | 상태 −1000 다수 | 20묶음 |
| neos5 VB3 | 실제 | 상태 2·−1000 | 33묶음 |
| neos-3627168-kasai VB3 | 실제 | 상태 3·6, down ≥ 1e100 | 27묶음, 상태 3 23·6 1, down ≥ 1e100 24 |
| 합성 `direct_requests_and_restores_own_bound` | 합성 | 문턱 밖 ∓1e100·배율·−0 | Rust 단위 테스트 |

tr12-30 VB3 NodeLimit 20은 훅을 단 채 300초를 넘겨 탐색에서 뺐다.

## 5. 재실행 명령

- 캡처: `tools/hrun.sh --timeout 580000 -- 'tools/mip_sb_direct_runs.sh bench/oracle/m2_06h6/final'`(Frida 위치가 다르면 `FRIDA_NODE`).
- 재생·비교·판별: `python3 tools/mip_sb_loop_rust.py --self-test`(manifest 갱신 `--write-manifest`).
- 회귀: `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`. Python 확장 미변경이면 pytest 제외.

## 6. 실행 결과

- 실행 commit `f6b5dfc`(도구·자료 해시는 `bench/oracle/m2_06h6/final/manifest.json`, 06h-3·06h-4 자료까지 함께 묶는다. `bench/oracle/m2_06h4/final/manifest.json`은 당시 기록으로 남는다). x86-64 Linux(WSL2), Node 22, Frida 17(이 컴퓨터는 `~/.local/share/frida-node`가 없어 `FRIDA_NODE`를 옛 npx 경로로 지정).
- 탐색(scratch, 기준 아님): 같은 도구로 markshare_4_0·pk1·neos5·kasai VB3 NodeLimit 20이 모두 `ok true`(06h-4의 접근 위반 2종 해소). tr12-30은 훅을 단 채 300초를 넘겨 중단했고, ran14x18-disj-8은 끝까지 보지 않았다.
- 변경: `strong_branch::Batch`의 `lower`·`upper`를 `bounds: Bounds`로 바꿨다. `Propagate { lower, upper }`는 기존 동작, `Direct { lower, upper, scale, infinity }`는 C:454–462·841–855 직접 요청과 C:583–586·974–990 무조건 복구(값 = C:410–427)다. 비전파에서는 6·개수 work를 더하지 않는다. 하네스는 커널 사건의 `propagation` 필드로 경로를 고르고, 비전파 묶음의 복구 값 입력을 `bounds0`·`input.scales`·`input.inf`로 만든다. 요청/복구 구분은 직전 단계가 `restore`인지로 한다(첫 실행에서 "다음이 flush면 요청"으로 했다가 마지막 후보의 down 복구 뒤 종료 flush를 요청으로 세어 원본 자료 단계에서 `direction order`로 멈췄다. Rust 실행 전이었다).
- 원본: `tools/mip_sb_direct_runs.sh bench/oracle/m2_06h6/final` 종료 0(15분 40초, 원본 17회). 4모델 각각 독립 3회 JSON 동일, 무훅 대조 최종 출력 동일. 비간섭: 기본 설정 pk1(`pk1_default`)에서 새 필드(`node`·`propagation`·`inf`)를 뺀 사건·출력·카운트가 `m2_06h3/final/pk1_1`과 같다(`controls()`).
- 재생: `python3 tools/mip_sb_loop_rust.py` 종료 0. **389묶음**(전파 289는 06h-5와 같은 수, 비전파 100: markshare_4_0 20·pk1 20·neos5 33·kasai 27) 명령 trace·최종 배열·반복 비용·work 비트 일치. 비전파 사례: 직접 요청 2,501·복구 2,501, 상태 2 988·−1000 1,489·3 23·6 1, values 904, down ≥ 1e100 24, 후보 통째 건너뛰기 8, down 게이트 69, up 사전 채움 30, down 사전 채움 80, 사전 채움 값 12. 필수 7종 모두 0보다 크다.
- 오류 검출: `--self-test` 종료 0([로그](../../bench/oracle/m2_06h6/selftest.log)). 거부 54: 기존 변조 18·직접 3·trace 4(기존 전파 자료에만 적용), 새 변조 5(`direct flag`·`direct inf`·`direct scale`·`direct bounds`·`no direct`, VB3 자료에만 적용), Rust 변이 27 중 24(새 6종 `direct restore drop`·`restore gated`·`work`·`side`·`scale`·`request` 모두 거부). 생존 3은 계약에 미리 적은 목록과 같다: `restore strict`(합성 거부), `score shift`(관측 불가), `direct clamp`(이진 후보뿐이라 원본 사례 없음, 합성 `direct_requests_and_restores_own_bound`가 거부). 로그의 `direct values`·`direct stall`·`direct work`는 06h-3부터 있던 "builder 직접 변조"의 이름이다(비전파와 무관).
- 회귀: `cargo test --release --workspace` 156 통과·0 실패·12 무시([로그](../../bench/oracle/m2_06h6/cargo.log)), `cargo clippy --workspace --all-targets -- -D warnings` 경고 0. Python 확장 미변경으로 pytest 제외.
- 계약 변경(첫 Rust 재생 전): §3 Rust 변이 목록에서 `direct as prop`을 `direct request`로 바꾸고 `direct clamp`을 생존 예상으로 옮겼다(자료 확인: 후보가 모두 이진).

## 7. 판정과 인수인계

- 작업 수행: **완료(오라클 + 구현 L1)**. 동등성: 비전파 경로의 직접 요청·무조건 복구·복구 값·work 미가산은 **원본 자연 사례 범위 통과**(VarBranch=3 4모델 100묶음, 오차 0). 06h-3·06h-4·06h-5 범위 289묶음 회귀 통과.
- 미검증: 뒤집힌 열 복구 값, `'C'` 후보, 문턱 밖 bound(합성만), 비전파의 상태 7·11·타이머 만료·up ≥ 1e100·stall 종료, `62a05d` 비전파 블록 진입 조건의 배타성, Threads > 1. LP 수치·B&B 미연결.
- 후속: **M2-06h-7**(조사: 전파 모순·연속 열 후보 탐색, 이번에 분리), **M2-06h-8**(타이머 산술, 06h-5의 가칭 06h-7에서 번호 이동), M2-06i(reliability 후보 판정).
- branch `work/m2-06h-6`, worktree `../rsolver-m2-06h-6`. 진행 `docs/progress/M2-06h-6.md`.
- 교훈: `docs/lessons/m2_sb_snapshot.md`(비전파 복구·요청 구분), `docs/lessons/tooling.md`(Frida 경로, 해시 고정 도구 위 치환 도구).
