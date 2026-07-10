"""Python bindings tests for spyder."""

import pytest

spyder = pytest.importorskip("spyder")
Robot = spyder.Robot


def test_rect_ik_four_lengths():
    r = Robot.rect(10, 6, 8)
    lengths = r.ik(0, 0, 2)
    assert len(lengths) == 4


def test_classify_rrpm():
    r = Robot.rect(10, 6, 8)
    assert r.classify() == "RRPM"


def test_polygon_six_cables():
    r = Robot.polygon(6, 5, 8)
    assert len(r.ik(0, 0, 2)) == 6


def test_pulley_model_increases_lengths():
    r = Robot.rect(10, 6, 8)
    ideal = r.ik(0.2, 0.0, 2.0)
    r.set_model("pulley", pulley_radius=0.05)
    assert r.model() == "pulley"
    pulley = r.ik(0.2, 0.0, 2.0)
    assert all(p >= i for p, i in zip(pulley, ideal))


def test_invalid_model_raises():
    r = Robot.rect(4, 4, 3)
    with pytest.raises(ValueError):
        r.set_model("bogus")


def test_fk_round_trip():
    r = Robot.rect(10, 6, 8)
    lengths = r.ik(0.1, -0.1, 2.0)
    x, y, z, residual, _method = r.fk(lengths, 0, 0, 2)
    assert abs(z - 2.0) < 0.05
    assert residual < 0.01


def test_jacobian_shape():
    r = Robot.rect(10, 6, 8)
    j = r.jacobian(0, 0, 2)
    assert len(j) == 4
    assert all(len(row) == 3 for row in j)


def test_workspace_fraction_positive():
    r = Robot.rect(10, 6, 8)
    frac = r.workspace_fraction(-2, 2, -2, 2, 0.5, 4, 5, 5, 4)
    assert 0.0 < frac <= 1.0


def test_ik_tensions_positive():
    r = Robot.rect(10, 6, 8)
    t = r.ik_tensions(0, 0, 2)
    assert len(t) == 4
    assert all(x > 0 for x in t)
