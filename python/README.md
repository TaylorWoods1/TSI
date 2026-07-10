# Spyder Python bindings

PyO3 extension exposing `spyder.Robot` — a thin wrapper over `spyder-core` and `spyder-sim`.

## Install (development)

```bash
cd python
python3 -m venv .venv && source .venv/bin/activate
pip install -U pip maturin==1.4.0 pytest
maturin develop --release
```

## Quick example

```python
from spyder import Robot

r = Robot.rect(10, 6, 8)
lengths = r.ik(0, 0, 2)
print(r.classify())          # "RRPM"
print(r.is_feasible(0, 0, 2))
tensions = r.ik_tensions(0, 0, 2)

r.set_model("pulley", pulley_radius=0.05)
x, y, z, residual, method = r.fk(lengths, 0, 0, 2)

j = r.jacobian(0, 0, 2)      # 4×3 nested lists
frac = r.workspace_fraction(-2, 2, -2, 2, 0.5, 4, 5, 5, 4)
```

## API reference

| Method | Description |
|--------|-------------|
| `Robot.rect(width, depth, height)` | 4-corner rectangle preset |
| `Robot.polygon(n, radius, height)` | Regular n-gon preset |
| `ik(x, y, z)` | Cable lengths (m) |
| `ik_tensions(x, y, z, mg=9.81)` | Lengths + tensions under gravity |
| `ik_with_wrench(x, y, z, mg, f_min, f_max)` | Tuple `(lengths, tensions)` |
| `is_feasible(x, y, z, mg=9.81)` | Wrench feasibility bool |
| `fk(lengths, seed_x, seed_y, seed_z)` | `(x, y, z, residual, method)` |
| `jacobian(x, y, z)` | Length Jacobian rows |
| `workspace_fraction(xmin, xmax, …)` | Feasible fraction in box |
| `line_ik(x0,y0,z0, x1,y1,z1, segments)` | IK along Cartesian line |
| `classify()` | `"IRPM"` / `"CRPM"` / `"RRPM"` |
| `set_model("ideal"\|"pulley"\|"sag", …)` | Cable model |
| `model()` | Current model name |

Docstrings are generated from `python/src/lib.rs` via PyO3 `///` comments.

## Notebook

```bash
jupyter notebook ../notebooks/01_ik_workspace.ipynb
```

## Tests

```bash
pytest tests/ -q
```

Nine tests cover rect/polygon layouts, pulley model, FK round-trip, jacobian shape, workspace, and error handling.

## Build note

The package name on PyPI/import is `spyder`. Source lives in `python/` and is **excluded** from the root Cargo workspace; maturin links against workspace crates via `python/Cargo.toml`.
