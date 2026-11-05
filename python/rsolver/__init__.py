"""gurobipy-shaped Python layer over the Rust core (re/notes/01: Python -> extension -> engine)."""
from ._core import INF, solve_problem


class GRB:
    MINIMIZE, MAXIMIZE = 1, -1
    CONTINUOUS, INTEGER, BINARY = "C", "I", "B"
    OPTIMAL, INFEASIBLE, UNBOUNDED = 2, 3, 5
    INFINITY = INF


class LinExpr:
    def __init__(self, terms=None, const=0.0):
        self.terms = dict(terms or {})  # Var -> coef
        self.const = const

    def __add__(self, o):
        if isinstance(o, (int, float)):
            return LinExpr(self.terms, self.const + o)
        o = _lin(o)
        t = dict(self.terms)
        for v, c in o.terms.items():
            t[v] = t.get(v, 0.0) + c
        return LinExpr(t, self.const + o.const)

    __radd__ = __add__

    def __neg__(self):
        return self * -1

    def __sub__(self, o):
        return self + (-_lin(o))

    def __rsub__(self, o):
        return _lin(o) - self

    def __mul__(self, k):
        return LinExpr({v: c * k for v, c in self.terms.items()}, self.const * k)

    __rmul__ = __mul__

    def __le__(self, rhs):
        return TempConstr(self - rhs, -INF, 0.0)

    def __ge__(self, rhs):
        return TempConstr(self - rhs, 0.0, INF)

    def __eq__(self, rhs):
        return TempConstr(self - rhs, 0.0, 0.0)

    __hash__ = object.__hash__


class Var(LinExpr):
    def __init__(self, model, index, name):
        super().__init__({}, 0.0)
        self.terms = {self: 1.0}
        self._model, self.index, self.VarName = model, index, name

    @property
    def X(self):
        return self._model._x[self.index]

    def __repr__(self):
        return f"<Var {self.VarName}>"


def _lin(o):
    return o if isinstance(o, LinExpr) else LinExpr({}, float(o))


class TempConstr:
    def __init__(self, expr, lo, hi):
        self.expr, self.lo, self.hi = expr, lo, hi


class Constr:
    def __init__(self, index, name):
        self.index, self.ConstrName = index, name


class Model:
    def __init__(self, name=""):
        self.ModelName = name
        self._vars, self._rows = [], []  # rows: ([(col, coef)...], lo, hi)
        self._obj, self.ModelSense = LinExpr(), GRB.MINIMIZE
        self.Status, self._x, self._objval = None, [], None

    def addVar(self, lb=0.0, ub=INF, obj=0.0, vtype=GRB.CONTINUOUS, name=""):
        if vtype == GRB.BINARY:
            lb, ub = max(lb, 0.0), min(ub, 1.0)
        v = Var(self, len(self._vars), name or f"x{len(self._vars)}")
        v.lb, v.ub, v.vtype = lb, ub, vtype
        self._vars.append(v)
        if obj:
            self._obj = self._obj + obj * v
        return v

    def addVars(self, *dims, **kw):
        from itertools import product
        keys = list(product(*(range(d) if isinstance(d, int) else list(d) for d in dims)))
        name = kw.pop("name", "x")
        return {k if len(k) > 1 else k[0]: self.addVar(name=f"{name}[{','.join(map(str, k))}]", **kw) for k in keys}

    def addConstr(self, tc, name=""):
        if not isinstance(tc, TempConstr):
            raise TypeError("addConstr expects expr <= / >= / == rhs")
        coefs = [(v.index, c) for v, c in tc.expr.terms.items() if c]
        self._rows.append((coefs, tc.lo - tc.expr.const, tc.hi - tc.expr.const))
        return Constr(len(self._rows) - 1, name)

    def setObjective(self, expr, sense=GRB.MINIMIZE):
        self._obj, self.ModelSense = _lin(expr), sense

    def optimize(self):
        n = len(self._vars)
        obj = [0.0] * n
        for v, c in self._obj.terms.items():
            obj[v.index] += c * self.ModelSense
        self.Status, val, self._x = solve_problem(
            obj, self._rows, [v.lb for v in self._vars], [v.ub for v in self._vars],
            [v.vtype != GRB.CONTINUOUS for v in self._vars])
        self._objval = val * self.ModelSense + self._obj.const

    @property
    def ObjVal(self):
        if self.Status != GRB.OPTIMAL:
            raise AttributeError("no solution available")
        return self._objval

    def getVars(self):
        return list(self._vars)


def quicksum(it):
    e = LinExpr()
    for t in it:
        e = e + t
    return e
