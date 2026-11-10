"""형식 검사기의 합성 오류 주입. 이 fixture는 원본 증거가 아니다."""
from copy import deepcopy
from hashlib import sha256
from pathlib import Path
from crossover_entry_check import EXPECTED, check, difference

bits = "0000000000000000"
state = {"model": 1, "n": 1, "m": 1, "status": 2,
         "env": {"field_2478": [bits], "field_24a0": [bits], "field_26fc": 0},
         "scale": {"c": None, "r": None, "o": [bits], "flip": None, "flip_enabled": 0, "sense": ["3e"]},
         "result": {k: [bits] for k in ("x", "slack", "reduced", "y")}}
state["copy"] = {"primal": [bits, "0000000000000080"], "dual": [bits, bits]}
leave = deepcopy(state)
leave["workspace"] = {"rows": 1, "cols": 1, "vstat": ["00000000", "00000000"]}
prepared = {"event": "prepared", "model": 1, **{k: [bits] for k in ("x", "slack", "reduced", "y")}}
d = {"origin": "natural", "kind": "barrier",
     "model_sha256": sha256(Path("bench/oracle/m3_crossover_entry/small.lp").read_bytes()).hexdigest(),
     "binary_sha256": EXPECTED, "tool_sha256": sha256(Path("tools/crossover_entry_capture.js").read_bytes()).hexdigest(),
     "params": {"Threads": 1, "Seed": 0, "OutputFlag": 0, "Presolve": 0, "Method": 2, "Crossover": 1, "LPWarmStart": 2},
     "starts": None, "result": {"status": 2}, "errors": [],
     "events": [{"event": "barrier:enter", "call": 1, "state": state},
                {"event": "dispatch:enter", "call": 2, "state": state},
                {"event": "body:enter", "call": 3, "mode": 0, "state": state}, prepared,
                {"event": "body:leave", "call": 3, "ret": 0, "state": leave},
                {"event": "dispatch:leave", "call": 2, "state": state},
                {"event": "barrier:leave", "call": 1, "state": state}]}
assert check(d) == 1
for mutate in (lambda x: x.update(model_sha256="0" * 64), lambda x: x.update(model_sha256="f" * 64),
               lambda x: x.update(binary_sha256="0" * 64), lambda x: x.update(events=[]),
               lambda x: x.update(origin="synthetic"), lambda x: x["events"].reverse(),
               lambda x: x["events"][2]["state"]["result"].update(x=None),
               lambda x: x["events"][2]["state"].update(scale={"sense": []}),
               lambda x: x["events"][3].update(x=[])):
    bad = deepcopy(d)
    mutate(bad)
    try:
        check(bad)
    except ValueError:
        continue
    raise AssertionError("변조를 놓쳤음")
bad = deepcopy(d)
bad["events"][4]["ret"] = 3
assert "ret" in difference(d, bad, case="시험")
print("합성 오류 주입 10건 검출; 원본 성공 아님")
