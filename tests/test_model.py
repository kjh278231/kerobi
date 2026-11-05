import pytest
from rsolver import GRB, Model, quicksum


def test_lp_max():
    m = Model()
    x, y = m.addVar(name="x"), m.addVar(name="y")
    m.addConstr(x + y <= 4)
    m.addConstr(x + 3 * y <= 6)
    m.setObjective(3 * x + 2 * y, GRB.MAXIMIZE)
    m.optimize()
    assert m.Status == GRB.OPTIMAL
    assert m.ObjVal == pytest.approx(12)
    assert (x.X, y.X) == pytest.approx((4, 0))


@pytest.mark.xfail(raises=Exception, strict=True, reason="MIP engine is ported in M2; the textbook core was removed in M1")
def test_mip_knapsack():
    m = Model()
    v = m.addVars(3, vtype=GRB.BINARY)
    w, p = [2, 3, 1], [5, 4, 3]
    m.addConstr(quicksum(w[i] * v[i] for i in range(3)) <= 5)
    m.addConstr(4 * v[0] + v[1] + 2 * v[2] <= 11)
    m.addConstr(3 * v[0] + 4 * v[1] + 2 * v[2] <= 8)
    m.setObjective(quicksum(p[i] * v[i] for i in range(3)), GRB.MAXIMIZE)
    m.optimize()
    assert m.Status == GRB.OPTIMAL
    assert m.ObjVal == pytest.approx(9)
    assert [v[i].X for i in range(3)] == [1, 1, 0]


def test_infeasible():
    m = Model()
    x = m.addVar(ub=1)
    m.addConstr(x >= 2)
    m.optimize()
    assert m.Status == GRB.INFEASIBLE
    with pytest.raises(AttributeError):
        m.ObjVal
