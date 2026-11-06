"""gurobipy-shaped Python layer over the Rust core (re/notes/01: Python -> extension -> engine)."""
from ._core import INF, build_model, read_model


class GRB:
    MINIMIZE, MAXIMIZE = 1, -1
    CONTINUOUS, INTEGER, BINARY = "C", "I", "B"
    OPTIMAL, INFEASIBLE, UNBOUNDED = 2, 3, 5
    INFINITY = INF

    class Status:
        LOADED, OPTIMAL, INFEASIBLE, INF_OR_UNBD, UNBOUNDED, CUTOFF, ITERATION_LIMIT, NODE_LIMIT = 1, 2, 3, 4, 5, 6, 7, 8
        TIME_LIMIT, SOLUTION_LIMIT, INTERRUPTED, NUMERIC, SUBOPTIMAL, INPROGRESS, USER_OBJ_LIMIT = 9, 10, 11, 12, 13, 14, 15
        WORK_LIMIT, MEM_LIMIT, LOCALLY_OPTIMAL, LOCALLY_INFEASIBLE = 16, 17, 18, 19

    class Callback:  # same codes as gurobipy
        POLLING, PRESOLVE, SIMPLEX, MIP, MIPSOL, MIPNODE, MESSAGE, BARRIER, MULTIOBJ, IIS, PDHG = range(11)
        PRE_COLDEL, PRE_ROWDEL, PRE_SENCHG, PRE_BNDCHG, PRE_COECHG = 1000, 1001, 1002, 1003, 1004
        SPX_ITRCNT, SPX_OBJVAL, SPX_PRIMINF, SPX_DUALINF, SPX_ISPERT = 2000, 2001, 2002, 2003, 2004
        MIP_OBJBST, MIP_OBJBND, MIP_NODCNT, MIP_SOLCNT, MIP_CUTCNT, MIP_NODLFT, MIP_ITRCNT = 3000, 3001, 3002, 3003, 3004, 3005, 3006
        MIPSOL_SOL, MIPSOL_OBJ, MIPSOL_OBJBST, MIPSOL_OBJBND, MIPSOL_NODCNT, MIPSOL_SOLCNT = 4001, 4002, 4003, 4004, 4005, 4006
        MIPNODE_STATUS, MIPNODE_REL, MIPNODE_OBJBST, MIPNODE_OBJBND, MIPNODE_NODCNT, MIPNODE_SOLCNT = 5001, 5002, 5003, 5004, 5005, 5006
        BARRIER_ITRCNT, BARRIER_PRIMOBJ, BARRIER_DUALOBJ, BARRIER_PRIMINF, BARRIER_DUALINF, BARRIER_COMPL = 7001, 7002, 7003, 7004, 7005, 7006
        PDHG_ITRCNT, PDHG_PRIMOBJ, PDHG_DUALOBJ, PDHG_PRIMINF, PDHG_DUALINF, PDHG_COMPL = 10001, 10002, 10003, 10004, 10005, 10006


class GurobiError(Exception):
    def __init__(self, errno, message):
        super().__init__(message)
        self.errno, self.message = errno, message


class Env:
    def __init__(self, logfilename="", empty=False, params=None):
        self.params = dict(params or {})

    def setParam(self, name, value):
        self.params[name] = value

    def start(self):
        return self

    def dispose(self):
        pass


def read(filename, env=None):
    """Read an MPS file (01 §1.5) into a model; arrays live in the Rust core."""
    m = Model()
    try:
        m._core = read_model(str(filename))
    except ValueError as e:
        raise GurobiError(10012, str(e)) from None
    m.ModelName, m.ModelSense = m._core.name, m._core.sense
    m.Params = dict(env.params) if env else {}
    return m


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
        self._core, self.Params = None, {}

    # Attributes of a model read from a file (01 §1.4).
    @property
    def Fingerprint(self):
        return self._core.fingerprint()

    @property
    def NumVars(self):
        return len(self._core.obj) if self._core else len(self._vars)

    @property
    def NumConstrs(self):
        return len(self._core.rhs) if self._core else len(self._rows)

    @property
    def NumNZs(self):
        return self._core.num_nz if self._core else sum(len(r[0]) for r in self._rows)

    @property
    def IsMIP(self):
        vt = self._core.vtype if self._core else "".join(v.vtype for v in self._vars)
        return int(any(t != "C" for t in vt))

    @property
    def ObjCon(self):
        return self._core.objcon if self._core else self._obj.const

    def setParam(self, name, value):
        self.Params[name] = value

    def presolve(self):
        if self._core is None:
            raise GurobiError(10024, "presolve of a model built in Python is not ported yet (M1)")
        q = Model()
        try:
            q._core = self._core.presolve([(k, float(v)) for k, v in self.Params.items() if isinstance(v, (int, float))])
        except RuntimeError as e:  # presolve proved the MIP infeasible: gurobipy raises with errno -1
            raise GurobiError(-1, str(e)) from None
        q.ModelName, q.ModelSense, q.Params = q._core.name, q._core.sense, dict(self.Params)
        return q

    def write(self, filename):
        if self._core is None or not str(filename).endswith((".mps", ".mps.gz")):
            raise GurobiError(10024, "only MPS writing of a model read from a file is ported (01 §1.5)")
        text = self._core.to_mps().encode()
        if str(filename).endswith(".gz"):
            import gzip
            with gzip.open(filename, "wb") as f:
                f.write(text)
        else:
            with open(filename, "wb") as f:
                f.write(text)

    def dispose(self):
        self._core = None

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

    def _param(self, name, default):
        for k, v in self.Params.items():
            if k.lower() == name.lower():
                return v
        return default

    def optimize(self, callback=None):
        if self._core is not None and self.Params.get("Method") == 2:
            return self._barrier(callback)
        if self._core is not None and self.Params.get("Method") == 6:
            return self._pdhg(callback)
        core, built = self._core, self._core is None
        if built:  # a model built through the API: same engine, minimization form, ranged rows as slack columns
            obj = [0.0] * len(self._vars)
            for v, c in self._obj.terms.items():
                obj[v.index] += c * self.ModelSense
            core = build_model(obj, self._rows, [v.lb for v in self._vars], [v.ub for v in self._vars],
                               "".join("C" if v.vtype == GRB.CONTINUOUS else "I" for v in self._vars))
        if any(t != "C" for t in core.vtype):
            raise GurobiError(10024, "MIP engine is not ported yet (M2)")
        import time
        t0 = time.perf_counter()
        st, obj, x, it, trace, self._path = core.solve(int(self._param("Method", -1)), float(self._param("IterationLimit", 1e100)),
                                                       callback is not None)
        self.Runtime = time.perf_counter() - t0
        self.Status, self.IterCount = st, it
        self._x = x[:len(self._vars)] if built else x
        self._objval = obj * self.ModelSense + self._obj.const if built else obj
        self.BarIterCount, self.NodeCount, self.SolCount, self.Work = 0, 0.0, int(st == GRB.OPTIMAL), 0.0
        if callback is not None:  # SIMPLEX callbacks replayed in iteration order after the solve
            for row in trace:
                self._cbrow = row
                callback(self, GRB.Callback.SIMPLEX)
            self._cbrow = None

    def cbGet(self, what):
        if getattr(self, "_cbrow", None) is None:  # BARRIER callback (set by _barrier)
            return self._cbdata[what]
        k = {GRB.Callback.SPX_ITRCNT: 0, GRB.Callback.SPX_OBJVAL: 1, GRB.Callback.SPX_PRIMINF: 2,
             GRB.Callback.SPX_DUALINF: 3, GRB.Callback.SPX_ISPERT: 4}[what]
        v = self._cbrow[k]
        return int(v) if k == 4 else v

    @property
    def ObjBound(self):
        return self.ObjVal

    def _barrier(self, callback):
        """M3 barrier (Presolve=0, Crossover=0 assumed): BARRIER callback (where = 7) once per iteration."""
        import time

        C = GRB.Callback
        keys = (C.BARRIER_ITRCNT, C.BARRIER_PRIMOBJ, C.BARRIER_DUALOBJ, C.BARRIER_PRIMINF, C.BARRIER_DUALINF,
                C.BARRIER_COMPL)

        def cb(*vals):
            if callback:
                self._cbdata = dict(zip(keys, vals))
                callback(self, C.BARRIER)

        p, t0 = self.Params, time.perf_counter()
        if p.get("LogFile"):  # the "Barrier statistics" block (2e8590) that tools/parity.py compares
            _, aat, nz, ops, free = self._core.barrier_symbolic()
            with open(p["LogFile"], "a") as f:
                f.write("\nBarrier statistics:\n" + (f" Free vars  : {free}\n" if free else "")
                        + f" AA' NZ     : {aat / 2:.3e}\n Factor NZ  : {nz:.3e}\n Factor Ops : {ops:.3e}\n Threads    : 1\n\n")
        self.Status, self._objval, self.BarIterCount, self._x, self.Work = self._core.barrier(
            cb, float(p.get("BarConvTol", 1e-8)), int(p.get("BarIterLimit", 1000)), int(p.get("BarCorrectors", -1)))
        self.Runtime, self.IterCount, self.NodeCount = time.perf_counter() - t0, 0, 0

    def _pdhg(self, callback):
        """M3 PDHG (Presolve=0, Crossover=0 assumed): PDHG callback (where = 10) on every evaluation pass."""
        import time

        C = GRB.Callback
        keys = (C.PDHG_ITRCNT, C.PDHG_PRIMOBJ, C.PDHG_DUALOBJ, C.PDHG_PRIMINF, C.PDHG_DUALINF, C.PDHG_COMPL)

        def cb(*vals):
            if callback:
                self._cbdata = dict(zip(keys, vals))
                callback(self, C.PDHG)

        t0 = time.perf_counter()
        prm = [float(self._param(k, d)) for k, d in (("PDHGIterLimit", 1e100), ("PDHGRelTol", 1e-6),
                                                     ("PDHGAbsTol", 1e-6), ("PDHGConvTol", 1e-6), ("WorkLimit", 1e100))]
        self.Status, self._objval, it, self._x, self.Work = self._core.pdhg(cb, *prm)
        self.PDHGIterCount, self.SolCount = float(it), 1  # a PDHG iterate is a solution even at a limit
        self.Runtime, self.IterCount, self.BarIterCount, self.NodeCount = time.perf_counter() - t0, 0.0, 0, 0.0

    @property
    def ObjVal(self):
        if self.Status != GRB.OPTIMAL and not getattr(self, "SolCount", 0):
            raise AttributeError("no solution available")
        return self._objval

    def getVars(self):
        return list(self._vars)


def quicksum(it):
    e = LinExpr()
    for t in it:
        e = e + t
    return e
