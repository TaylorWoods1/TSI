import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";
import * as api from "../api/client";
import { useApp } from "../context";
import RobotScene, { type SceneData } from "../scene/RobotScene";

function Collapsible({
  title,
  children,
  badge,
}: {
  title: string;
  children: ReactNode;
  badge?: ReactNode;
}) {
  return (
    <details
      style={{
        border: "1px solid var(--border)",
        borderRadius: 6,
        padding: "0.45rem 0.5rem",
      }}
    >
      <summary
        style={{
          cursor: "pointer",
          color: "var(--text-dim)",
          fontSize: "0.75rem",
          fontWeight: 600,
          userSelect: "none",
          display: "flex",
          alignItems: "center",
          gap: "0.5rem",
        }}
      >
        {title}
        {badge}
      </summary>
      <div style={{ marginTop: "0.5rem" }}>{children}</div>
    </details>
  );
}

function ClassifyBadge({ value }: { value: string }) {
  return (
    <span
      style={{
        display: "inline-block",
        padding: "0.15rem 0.45rem",
        borderRadius: 4,
        background: "rgba(61, 214, 198, 0.15)",
        border: "1px solid var(--border)",
        color: "var(--accent)",
        fontFamily: "var(--font-mono)",
        fontSize: "0.7rem",
        fontWeight: 700,
      }}
    >
      {value}
    </span>
  );
}

function SliderField({
  label,
  value,
  min,
  max,
  step,
  onChange,
}: {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  onChange: (v: number) => void;
}) {
  return (
    <div className="field">
      <label>
        {label}: {value.toFixed(2)}
      </label>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(parseFloat(e.target.value))}
        style={{ width: "100%", accentColor: "var(--accent)" }}
      />
    </div>
  );
}

const DEFAULT_WS = {
  min: [-2, -2, 0.5] as [number, number, number],
  max: [2, 2, 4] as [number, number, number],
  nx: 7,
  ny: 7,
  nz: 5,
  mg: 50,
  f_min: 0.5,
  f_max: 500,
};

export default function SimulatePage() {
  const {
    venue,
    classify,
    dolly,
    setDolly,
    orientationRv,
    setOrientationRv,
    traj,
    setTraj,
    scene,
    sceneLoading,
    workspacePts,
    setWorkspacePts,
    showPulls,
    setShowPulls,
    fkResidual,
    setFkResidual,
  } = useApp();

  const [error, setError] = useState<string | null>(null);
  const [playing, setPlaying] = useState(false);
  const [waypoints, setWaypoints] = useState<[number, number, number][]>([]);
  const [lengths, setLengths] = useState<number[][]>([]);
  const [wsCfg, setWsCfg] = useState(DEFAULT_WS);
  const [wsLoading, setWsLoading] = useState(false);

  const [ikText, setIkText] = useState("—");
  const [fkText, setFkText] = useState("—");
  const [jacRows, setJacRows] = useState<number[][]>([]);
  const [feasibleOk, setFeasibleOk] = useState<boolean | null>(null);
  const [currentLengths, setCurrentLengths] = useState<number[]>([]);
  const [referenceLengths, setReferenceLengths] = useState<number[]>([]);

  const frameRef = useRef(0);
  const animRef = useRef<number | null>(null);
  const analysisTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    api.getCalibration()
      .then((cal) => {
        if (cal.home_lengths_m.length > 0) {
          setReferenceLengths(cal.home_lengths_m);
        }
      })
      .catch(() => {});
  }, [venue?.anchors.length]);

  const refreshAnalysis = useCallback(async () => {
    if (!venue) return;
    const xyz: [number, number, number] = [dolly.x, dolly.y, dolly.z];
    const ori = !venue.point_mass ? orientationRv : undefined;
    try {
      setError(null);
      const ik = await api.ik(xyz, {
        mg: venue.model === "sag" || wsCfg.mg > 0 ? wsCfg.mg : undefined,
        model: venue.model,
        reference_lengths:
          referenceLengths.length === venue.anchors.length
            ? referenceLengths
            : undefined,
      });
      setCurrentLengths(ik.lengths);
      const tensionStr = ik.tensions
        ? `\ntensions: ${ik.tensions.map((t) => t.toFixed(2)).join(", ")} N`
        : "";
      const unstrainedStr = ik.unstrained_lengths
        ? `\nunstrained: ${ik.unstrained_lengths
            .map((u) => (u == null ? "—" : u.toFixed(3)))
            .join(", ")} m`
        : "";
      const motorStr = ik.motor_commands
        ? `\nmotor steps: ${ik.motor_commands.map((c) => c.steps).join(", ")}`
        : "";
      setIkText(
        `lengths: ${ik.lengths.map((l) => l.toFixed(3)).join(", ")} m${tensionStr}${unstrainedStr}${motorStr}`,
      );

      const fk = await api.fk(ik.lengths, xyz, {
        orientation_rv: ori,
        tensions: ik.tensions,
        allow_underconstrained: false,
      });
      setFkResidual(fk.residual);
      setFkText(
        `xyz: [${fk.xyz.map((x) => x.toFixed(4)).join(", ")}]\nresidual: ${fk.residual.toExponential(2)}\nmethod: ${fk.method}`,
      );

      const jac = await api.jacobian(xyz, ori);
      setJacRows(jac.rows);

      const feas = await api.feasible(xyz, {
        mg: wsCfg.mg,
        f_min: wsCfg.f_min,
        f_max: wsCfg.f_max,
      });
      setFeasibleOk(feas.ok);
    } catch (e) {
      setError(String(e));
    }
  }, [venue, dolly, orientationRv, wsCfg, setFkResidual, referenceLengths]);

  useEffect(() => {
    if (analysisTimer.current) clearTimeout(analysisTimer.current);
    analysisTimer.current = setTimeout(() => {
      void refreshAnalysis();
    }, 120);
    return () => {
      if (analysisTimer.current) clearTimeout(analysisTimer.current);
    };
  }, [dolly.x, dolly.y, dolly.z, orientationRv, venue?.model, refreshAnalysis]);

  const planTraj = async () => {
    try {
      setError(null);
      const res = await api.trajLine({
        start: traj.start,
        end: traj.end,
        segments: traj.segments,
      });
      setWaypoints(res.waypoints);
      setLengths(res.lengths);
      frameRef.current = 0;
      if (res.waypoints.length > 0) {
        const [x, y, z] = res.waypoints[0];
        setDolly({ x, y, z });
      }
    } catch (e) {
      setError(String(e));
    }
  };

  const planWaypointList = async () => {
    if (waypoints.length < 2) {
      setError("Need at least 2 waypoints");
      return;
    }
    try {
      setError(null);
      const res = await api.trajWaypoints(waypoints);
      setWaypoints(res.waypoints);
      setLengths(res.lengths);
      frameRef.current = 0;
    } catch (e) {
      setError(String(e));
    }
  };

  const togglePlay = () => {
    if (playing) {
      setPlaying(false);
      if (animRef.current) cancelAnimationFrame(animRef.current);
      return;
    }
    if (waypoints.length === 0) {
      void planTraj().then(() => setPlaying(true));
    } else {
      setPlaying(true);
    }
  };

  useEffect(() => {
    if (!playing || waypoints.length === 0) return;
    let last = performance.now();
    const tick = (now: number) => {
      if (now - last > 120) {
        last = now;
        const idx = frameRef.current;
        if (idx >= waypoints.length) {
          frameRef.current = 0;
          setPlaying(false);
          return;
        }
        const [x, y, z] = waypoints[idx];
        setDolly({ x, y, z });
        frameRef.current = idx + 1;
      }
      animRef.current = requestAnimationFrame(tick);
    };
    animRef.current = requestAnimationFrame(tick);
    return () => {
      if (animRef.current) cancelAnimationFrame(animRef.current);
    };
  }, [playing, waypoints, setDolly]);

  const sampleWorkspace = async () => {
    setWsLoading(true);
    try {
      setError(null);
      const res = await api.workspace({
        min: wsCfg.min,
        max: wsCfg.max,
        nx: wsCfg.nx,
        ny: wsCfg.ny,
        nz: wsCfg.nz,
        mg: wsCfg.mg,
        f_min: wsCfg.f_min,
        f_max: wsCfg.f_max,
      });
      const pts = res.samples
        .filter((s) => s.feasible)
        .map((s) => ({ x: s.x, y: s.y, z: s.z }));
      setWorkspacePts(pts);
    } catch (e) {
      setError(String(e));
    } finally {
      setWsLoading(false);
    }
  };

  const exportPlotly = async () => {
    try {
      setError(null);
      const res = await api.sceneExport({
        xyz: [dolly.x, dolly.y, dolly.z],
        format: waypoints.length > 1 ? "html_anim" : "html",
        waypoints: waypoints.length > 0 ? waypoints : undefined,
        orientation_rv: venue && !venue.point_mass ? orientationRv : undefined,
      });
      const blob = new Blob([res.html], { type: "text/html" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "spyder_scene.html";
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      setError(String(e));
    }
  };

  const updateWaypoint = (row: number, col: number, value: number) => {
    setWaypoints((wps) => {
      const next = wps.map((w) => [...w] as [number, number, number]);
      while (next.length <= row) next.push([0, 0, 2]);
      next[row][col] = value;
      return next;
    });
  };

  const addWaypoint = () => {
    setWaypoints((wps) => [...wps, [dolly.x, dolly.y, dolly.z]]);
  };

  const removeWaypoint = (row: number) => {
    setWaypoints((wps) => wps.filter((_, i) => i !== row));
  };

  const copyWaypoints = async () => {
    try {
      await navigator.clipboard.writeText(JSON.stringify(waypoints, null, 2));
    } catch (e) {
      setError(String(e));
    }
  };

  const pasteWaypoints = async () => {
    try {
      const text = await navigator.clipboard.readText();
      const parsed = JSON.parse(text) as [number, number, number][];
      if (!Array.isArray(parsed)) throw new Error("Expected JSON array");
      setWaypoints(parsed);
    } catch (e) {
      setError(String(e));
    }
  };

  const setTrajField = (key: "start" | "end", axis: number, value: number) => {
    const tuple = [...traj[key]] as [number, number, number];
    tuple[axis] = value;
    setTraj({ ...traj, [key]: tuple });
  };

  const displayScene: SceneData = scene ?? {
    anchors: venue?.anchors ?? [],
    dolly,
    attachments: venue?.attachments,
    lengths: lengths[Math.max(0, frameRef.current - 1)] ?? currentLengths,
    model: venue?.model,
    workspace: workspacePts,
    show_pulls: showPulls,
  };

  const dollyMin = -6;
  const dollyMax = 6;

  return (
    <div className="page">
      <div className="viewport">
        <RobotScene scene={displayScene} />
        {sceneLoading && (
          <div
            style={{
              position: "absolute",
              top: 8,
              left: 8,
              fontFamily: "var(--font-mono)",
              fontSize: "0.7rem",
              color: "var(--text-dim)",
            }}
          >
            Updating scene…
          </div>
        )}
      </div>
      <aside className="inspector">
        <h3>Simulate</h3>
        <div className="readout">
          Dolly: [{dolly.x.toFixed(2)}, {dolly.y.toFixed(2)}, {dolly.z.toFixed(2)}]
          {fkResidual != null && ` · FK residual ${fkResidual.toExponential(2)}`}
        </div>

        <Collapsible title="Pose scrubber">
          <SliderField
            label="X"
            value={dolly.x}
            min={dollyMin}
            max={dollyMax}
            step={0.05}
            onChange={(v) => setDolly({ ...dolly, x: v })}
          />
          <SliderField
            label="Y"
            value={dolly.y}
            min={dollyMin}
            max={dollyMax}
            step={0.05}
            onChange={(v) => setDolly({ ...dolly, y: v })}
          />
          <SliderField
            label="Z"
            value={dolly.z}
            min={0}
            max={12}
            step={0.05}
            onChange={(v) => setDolly({ ...dolly, z: v })}
          />
          {venue && !venue.point_mass && (
            <>
              <SliderField
                label="rx"
                value={orientationRv[0]}
                min={-3.14}
                max={3.14}
                step={0.05}
                onChange={(v) => setOrientationRv([v, orientationRv[1], orientationRv[2]])}
              />
              <SliderField
                label="ry"
                value={orientationRv[1]}
                min={-3.14}
                max={3.14}
                step={0.05}
                onChange={(v) => setOrientationRv([orientationRv[0], v, orientationRv[2]])}
              />
              <SliderField
                label="rz"
                value={orientationRv[2]}
                min={-3.14}
                max={3.14}
                step={0.05}
                onChange={(v) => setOrientationRv([orientationRv[0], orientationRv[1], v])}
              />
            </>
          )}
        </Collapsible>

        <label className="checkbox-row">
          <input
            type="checkbox"
            checked={showPulls}
            onChange={(e) => setShowPulls(e.target.checked)}
          />
          Show pull directions
        </label>

        <Collapsible title="Line plan">
          <div className="field">
            <label>Start XYZ</label>
            <div className="field-row">
              {traj.start.map((v, i) => (
                <input
                  key={i}
                  type="number"
                  step="0.1"
                  value={v}
                  onChange={(e) => setTrajField("start", i, parseFloat(e.target.value) || 0)}
                />
              ))}
            </div>
          </div>
          <div className="field">
            <label>End XYZ</label>
            <div className="field-row">
              {traj.end.map((v, i) => (
                <input
                  key={i}
                  type="number"
                  step="0.1"
                  value={v}
                  onChange={(e) => setTrajField("end", i, parseFloat(e.target.value) || 0)}
                />
              ))}
            </div>
          </div>
          <div className="field">
            <label>Segments</label>
            <input
              type="number"
              min={1}
              value={traj.segments}
              onChange={(e) =>
                setTraj({ ...traj, segments: parseInt(e.target.value, 10) || 1 })
              }
            />
          </div>
          <button type="button" className="btn" onClick={() => void planTraj()}>
            Plan line
          </button>
        </Collapsible>

        <Collapsible title="Waypoint editor">
          <div style={{ display: "flex", gap: "0.35rem", flexWrap: "wrap" }}>
            <button type="button" className="btn" onClick={addWaypoint}>
              Add row
            </button>
            <button type="button" className="btn" onClick={() => void copyWaypoints()}>
              Export JSON
            </button>
            <button type="button" className="btn" onClick={() => void pasteWaypoints()}>
              Import JSON
            </button>
            <button type="button" className="btn" onClick={() => void planWaypointList()}>
              Plan waypoints
            </button>
          </div>
          <div style={{ overflowX: "auto", marginTop: "0.5rem" }}>
            <table
              style={{
                width: "100%",
                borderCollapse: "collapse",
                fontFamily: "var(--font-mono)",
                fontSize: "0.7rem",
              }}
            >
              <thead>
                <tr style={{ color: "var(--text-dim)" }}>
                  <th style={{ textAlign: "left", padding: "0.2rem" }}>#</th>
                  <th>x</th>
                  <th>y</th>
                  <th>z</th>
                  <th />
                </tr>
              </thead>
              <tbody>
                {waypoints.map((wp, row) => (
                  <tr key={row}>
                    <td style={{ padding: "0.2rem" }}>{row + 1}</td>
                    {[0, 1, 2].map((col) => (
                      <td key={col} style={{ padding: "0.15rem" }}>
                        <input
                          type="number"
                          step="0.1"
                          value={wp[col]}
                          onChange={(e) =>
                            updateWaypoint(row, col, parseFloat(e.target.value) || 0)
                          }
                          style={{ width: "100%" }}
                        />
                      </td>
                    ))}
                    <td style={{ padding: "0.15rem" }}>
                      <button
                        type="button"
                        className="btn"
                        style={{ padding: "0.2rem 0.4rem", fontSize: "0.7rem" }}
                        onClick={() => removeWaypoint(row)}
                      >
                        ×
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          {waypoints.length === 0 && (
            <div className="readout" style={{ marginTop: "0.35rem" }}>
              No waypoints — plan a line or add rows.
            </div>
          )}
        </Collapsible>

        <button type="button" className="btn" onClick={togglePlay}>
          {playing ? "Pause" : "Play"}
        </button>

        <Collapsible title="IK">
          <div className="readout pre">{ikText}</div>
        </Collapsible>
        <Collapsible title="FK">
          <div className="readout pre">{fkText}</div>
        </Collapsible>
        <Collapsible title="Jacobian">
          {jacRows.length === 0 ? (
            <div className="readout">—</div>
          ) : (
            <div style={{ overflowX: "auto" }}>
              <table
                style={{
                  width: "100%",
                  borderCollapse: "collapse",
                  fontFamily: "var(--font-mono)",
                  fontSize: "0.65rem",
                }}
              >
                <tbody>
                  {jacRows.map((row, i) => (
                    <tr key={i}>
                      {row.map((v, j) => (
                        <td
                          key={j}
                          style={{
                            padding: "0.15rem 0.25rem",
                            borderBottom: "1px solid var(--border)",
                            color: "var(--text-dim)",
                          }}
                        >
                          {v.toFixed(4)}
                        </td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </Collapsible>
        <Collapsible title="Feasible">
          <div className="field-row">
            <div className="field">
              <label>mg (N)</label>
              <input
                type="number"
                step="1"
                value={wsCfg.mg}
                onChange={(e) =>
                  setWsCfg({ ...wsCfg, mg: parseFloat(e.target.value) || 0 })
                }
              />
            </div>
            <div className="field">
              <label>f_min</label>
              <input
                type="number"
                step="0.1"
                value={wsCfg.f_min}
                onChange={(e) =>
                  setWsCfg({ ...wsCfg, f_min: parseFloat(e.target.value) || 0 })
                }
              />
            </div>
            <div className="field">
              <label>f_max</label>
              <input
                type="number"
                step="1"
                value={wsCfg.f_max}
                onChange={(e) =>
                  setWsCfg({ ...wsCfg, f_max: parseFloat(e.target.value) || 0 })
                }
              />
            </div>
          </div>
          <div
            className="readout"
            style={{ color: feasibleOk ? "var(--accent)" : "var(--danger)" }}
          >
            {feasibleOk == null ? "—" : feasibleOk ? "Feasible" : "Infeasible"}
          </div>
        </Collapsible>
        <Collapsible title="Classify" badge={<ClassifyBadge value={classify} />}>
          <div className="readout">
            Robot classification: <ClassifyBadge value={classify} />
          </div>
        </Collapsible>

        <Collapsible title="Workspace overlay">
          <div className="field-row">
            {(["min", "max"] as const).map((bound) => (
              <div key={bound} className="field">
                <label>{bound} XYZ</label>
                <div className="field-row">
                  {wsCfg[bound].map((v, i) => (
                    <input
                      key={i}
                      type="number"
                      step="0.5"
                      value={v}
                      onChange={(e) => {
                        const arr = [...wsCfg[bound]] as [number, number, number];
                        arr[i] = parseFloat(e.target.value) || 0;
                        setWsCfg({ ...wsCfg, [bound]: arr });
                      }}
                    />
                  ))}
                </div>
              </div>
            ))}
          </div>
          <div className="field-row">
            {(["nx", "ny", "nz"] as const).map((k) => (
              <div key={k} className="field">
                <label>{k}</label>
                <input
                  type="number"
                  min={2}
                  value={wsCfg[k]}
                  onChange={(e) =>
                    setWsCfg({ ...wsCfg, [k]: parseInt(e.target.value, 10) || 2 })
                  }
                />
              </div>
            ))}
          </div>
          <button
            type="button"
            className="btn"
            disabled={wsLoading}
            onClick={() => void sampleWorkspace()}
          >
            {wsLoading ? "Sampling…" : "Sample workspace"}
          </button>
          <button type="button" className="btn" onClick={() => setWorkspacePts([])}>
            Clear overlay
          </button>
        </Collapsible>

        <button type="button" className="btn" onClick={() => void exportPlotly()}>
          Export Plotly HTML
        </button>

        {error && (
          <div className="readout" style={{ color: "var(--danger)" }}>
            {error}
          </div>
        )}
      </aside>
    </div>
  );
}
