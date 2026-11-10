"""형식 검사기의 합성 오류 주입. 이 fixture는 원본 증거가 아니다."""
from copy import deepcopy
from hashlib import sha256
from pathlib import Path
import struct
from crossover_entry_check import EXPECTED, check, difference, transformed

bits = "0000000000000000"
state = {"model": 1, "n": 1, "m": 1, "status": 2,
         "env": {"field_2478": [bits], "field_24a0": [bits], "field_26fc": 0},
         "scale": {"c": None, "r": None, "o": [bits], "flip": None, "flip_enabled": 0, "sense": ["3e"]},
         "result": {k: [bits] for k in ("x", "slack", "reduced", "y")}}
state["aliases"] = {k: i + 1 for i, k in enumerate(("result_x", "result_slack", "result_y", "result_reduced", "copy_primal", "copy_dual", "start_values", "scale_c", "scale_r", "flip", "sense", "vstat"))}
state["copy"] = {"primal": [bits, "0000000000000080"], "dual": [bits, bits]}
leave = deepcopy(state)
leave["workspace"] = {"rows": 1, "cols": 1, "vstat": ["00000000", "00000000"]}
prepared = {"event": "prepared", "model": 1, "scale": deepcopy(state["scale"]), **{k: [bits] for k in ("x", "slack", "reduced")}, "y": ["0000000000000080"]}
d = {"origin": "natural", "kind": "barrier",
     "model_sha256": sha256(Path("bench/oracle/m3_crossover_entry/small.lp").read_bytes()).hexdigest(),
     "binary_sha256": EXPECTED, "tool_sha256": sha256(Path("tools/crossover_entry_capture.js").read_bytes()).hexdigest(),
     "params": {"Threads": 1, "Seed": 0, "OutputFlag": 0, "Presolve": 0, "Method": 2, "Crossover": 1, "LPWarmStart": 2},
     "starts": None, "result": {"status": 2, "vbasis": None, "cbasis": None}, "errors": [],
     "events": [{"event": "barrier:enter", "call": 1, "state": state},
                {"event": "dispatch:enter", "call": 2, "state": state},
                {"event": "body:enter", "call": 3, "mode": 0, "state": state}, prepared,
                {"event": "body:leave", "call": 3, "ret": 0, "state": leave},
                {"event": "dispatch:leave", "call": 2, "state": state},
                {"event": "barrier:leave", "call": 1, "state": state}]}
assert check(d) == 1
for mutate in (lambda x: x.update(model_sha256="0" * 64), lambda x: x.update(model_sha256="f" * 64),
               lambda x: x.update(binary_sha256="0" * 64), lambda x: x.update(tool_sha256="f" * 64),
               lambda x: x["params"].update(Method=6), lambda x: x.update(events=[]),
               lambda x: x["events"][2].update(mode=1),
               lambda x: x["events"][2]["state"]["scale"].pop("c"),
               lambda x: x.update(origin="synthetic"), lambda x: x["events"].reverse(),
               lambda x: x["events"][2]["state"]["result"].update(x=None),
               lambda x: x["events"][2]["state"].update(scale={"sense": []}),
               lambda x: x["events"][3].update(x=[]),
               lambda x: x["events"][1]["state"]["copy"]["primal"].__setitem__(0, "0100000000000000")):
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
def f(v):
    return struct.pack("<d", v).hex()
r = {"x": [f(2)], "slack": [f(4)], "reduced": [f(8)], "y": [f(16)]}
scale = {"c": [f(2)], "r": [f(4)], "o": [f(2)], "flip": ["01"], "flip_enabled": 1, "sense": ["3e"]}
assert transformed(r, scale) == ([f(-4), f(-1)], [f(-2), f(32)])
zero = {k: [f(0.0)] for k in r}
scale.update(c=None, r=None, flip=None, flip_enabled=0, sense=["3c"])
assert transformed(zero, scale) == ([f(0.0), f(0.0)], [f(0.0), f(-0.0)])
print("합성 오류 주입 15건 및 스케일·부호·±0 확인; 원본 성공 아님")
