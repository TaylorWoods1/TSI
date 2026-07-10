import { useCallback, useEffect, useRef, useState } from "react";
import * as api from "../api/client";
import { useApp } from "../context";
import RobotScene, { type SceneData } from "../scene/RobotScene";

export default function SimulatePage() {
  const { venue, dolly, setDolly, traj, setTraj } = useApp();
  const [workspacePts, setWorkspacePts] = useState<api.Vec3[]>([]);
  const [readout, setReadout] = useState("");
  const [playing, setPlaying] = useState(false);
  const [waypoints, setWaypoints] = useState<[number, number, number][]>([]);
  const [lengths, setLengths] = useState<number[][]>([]);
  const frameRef = useRef(0);
  const animRef = useRef<number | null>(null);

  const updateReadout = useCallback(
    async (pos: api.Vec3) => {
      try {
        const ik = await api.ik([pos.x, pos.y, pos.z], 9.81);
        const tensionStr = ik.tensions
          ? `\ntensions: ${ik.tensions.map((t) => t.toFixed(2)).join(", ")}`
          : "";
        setReadout(
          `lengths: ${ik.lengths.map((l) => l.toFixed(3)).join(", ")}${tensionStr}`,
        );
      } catch (e) {
        setReadout(String(e));
      }
    },
    [],
  );

  useEffect(() => {
    updateReadout(dolly);
  }, [dolly, updateReadout]);

  const planTraj = async () => {
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
  };

  const togglePlay = () => {
    if (playing) {
      setPlaying(false);
      if (animRef.current) cancelAnimationFrame(animRef.current);
      return;
    }
    if (waypoints.length === 0) {
      planTraj().then(() => setPlaying(true));
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
    const res = await api.workspace({
      min: [-2, -2, 0.5],
      max: [2, 2, 4],
      nx: 7,
      ny: 7,
      nz: 5,
      mg: 9.81,
      f_min: 0.5,
      f_max: 500,
    });
    setWorkspacePts(
      res.samples.filter((s) => s.feasible).map((s) => ({ x: s.x, y: s.y, z: s.z })),
    );
    setReadout((r) => `${r}\nworkspace fraction: ${(res.fraction * 100).toFixed(1)}%`);
  };

  const scene: SceneData = {
    anchors: venue?.anchors ?? [],
    dolly,
    lengths: lengths[frameRef.current - 1] ?? [],
    workspace: workspacePts,
  };

  const setTrajField = (key: keyof typeof traj, axis: number, value: number) => {
    const copy = { ...traj };
    const tuple = [...copy[key === "segments" ? "start" : key]] as [number, number, number];
    if (key === "segments") {
      copy.segments = value;
    } else {
      tuple[axis] = value;
      copy[key] = tuple;
    }
    setTraj(copy);
  };

  return (
    <div className="page">
      <div className="viewport">
        <RobotScene scene={scene} />
      </div>
      <aside className="inspector">
        <h3>Simulate</h3>
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
        <button type="button" className="btn" onClick={planTraj}>
          Plan line
        </button>
        <button type="button" className="btn" onClick={togglePlay}>
          {playing ? "Pause" : "Play"}
        </button>
        <button type="button" className="btn" onClick={sampleWorkspace}>
          Workspace overlay
        </button>
        <div className="readout">{readout}</div>
      </aside>
    </div>
  );
}
