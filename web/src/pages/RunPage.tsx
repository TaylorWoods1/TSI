import { useEffect, useState } from "react";
import * as api from "../api/client";
import { useApp } from "../context";
import RobotScene, { type SceneData } from "../scene/RobotScene";

const DEFAULT_AXIS_MAP = `[
  { "board": 0, "motor": 0, "cable": 0 },
  { "board": 0, "motor": 1, "cable": 1 },
  { "board": 1, "motor": 0, "cable": 2 },
  { "board": 1, "motor": 1, "cable": 3 }
]`;

export default function RunPage() {
  const {
    venue,
    dolly,
    setDolly,
    traj,
    scene,
    sceneLoading,
    runBackend,
    setRunBackend,
  } = useApp();

  const [backend, setBackend] = useState("mock");
  const [device, setDevice] = useState("127.0.0.1:5555");
  const [baud, setBaud] = useState(115200);
  const [axisMapText, setAxisMapText] = useState(DEFAULT_AXIS_MAP);
  const [connected, setConnected] = useState(false);
  const [estopped, setEstopped] = useState(false);
  const [statusText, setStatusText] = useState("disconnected");
  const [closedLoop, setClosedLoop] = useState(false);
  const [realtime, setRealtime] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [safetyText, setSafetyText] = useState("—");
  const [playReadout, setPlayReadout] = useState("");

  useEffect(() => {
    if (!connected) return;
    const id = setInterval(async () => {
      try {
        const s = await api.runStatus();
        setConnected(s.connected);
        setEstopped(s.estopped);
        if (s.backend) setRunBackend(s.backend);
        if (s.pose) {
          setDolly({ x: s.pose[0], y: s.pose[1], z: s.pose[2] });
        }
        const steps = s.steps?.join(", ") ?? "—";
        setStatusText(
          `${s.backend ?? runBackend ?? "—"} | estop: ${s.estopped} | steps: [${steps}]`,
        );
        if (s.safety) {
          const { min, max, max_speed_mps } = s.safety;
          setSafetyText(
            `min: [${min.map((v) => v.toFixed(2)).join(", ")}]\nmax: [${max.map((v) => v.toFixed(2)).join(", ")}]\nmax speed: ${max_speed_mps.toFixed(2)} m/s`,
          );
        } else {
          setSafetyText("—");
        }
      } catch (e) {
        setError(String(e));
      }
    }, 250);
    return () => clearInterval(id);
  }, [connected, setDolly, setRunBackend, runBackend]);

  const parseAxisMap = (): object | undefined => {
    if (backend !== "multiboard") return undefined;
    try {
      return JSON.parse(axisMapText) as object;
    } catch {
      throw new Error("Invalid axis_map JSON");
    }
  };

  const connect = async () => {
    try {
      setError(null);
      const body: Parameters<typeof api.runConnect>[0] = { backend };
      if (backend === "stepper" || backend === "odrive") {
        body.device = device;
        body.baud = baud;
      } else if (backend === "multiboard") {
        body.axis_map = parseAxisMap();
      } else if (device) {
        body.device = device;
      }
      await api.runConnect(body);
      setConnected(true);
      setEstopped(false);
      setRunBackend(backend);
    } catch (e) {
      setError(String(e));
    }
  };

  const disconnect = async () => {
    try {
      setError(null);
      await api.runDisconnect();
      setConnected(false);
      setRunBackend(null);
      setStatusText("disconnected");
      setSafetyText("—");
    } catch (e) {
      setError(String(e));
    }
  };

  const home = async () => {
    try {
      setError(null);
      await api.runHome();
    } catch (e) {
      setError(String(e));
    }
  };

  const playLine = async () => {
    try {
      setError(null);
      const res = await api.runPlayLine({
        start: traj.start,
        end: traj.end,
        segments: traj.segments,
        closed_loop: closedLoop,
        realtime: realtime,
      });
      const poseStr = res.feedback_pose
        ? `\nfeedback: [${res.feedback_pose.map((v) => v.toFixed(3)).join(", ")}]`
        : "";
      setPlayReadout(
        `final_steps: [${res.final_steps.join(", ")}]${poseStr}`,
      );
    } catch (e) {
      setError(String(e));
    }
  };

  const triggerEstop = async () => {
    try {
      setError(null);
      await api.runEstop();
      setEstopped(true);
    } catch (e) {
      setError(String(e));
    }
  };

  const clearEstop = async () => {
    try {
      setError(null);
      await api.runClearEstop();
      setEstopped(false);
    } catch (e) {
      setError(String(e));
    }
  };

  const displayScene: SceneData = scene ?? {
    anchors: venue?.anchors ?? [],
    dolly,
    attachments: venue?.attachments,
    lengths: [],
    model: venue?.model,
  };

  const needsDevice = backend === "stepper" || backend === "odrive" || backend === "mock";
  const needsBaud = backend === "stepper" || backend === "odrive";
  const needsAxisMap = backend === "multiboard";

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
        {estopped && (
          <div
            className="danger"
            style={{
              position: "absolute",
              top: "50%",
              left: "50%",
              transform: "translate(-50%, -50%)",
              fontFamily: "var(--font-display)",
              fontSize: "1.5rem",
              fontWeight: 800,
              letterSpacing: "0.08em",
              pointerEvents: "none",
            }}
          >
            E-STOP
          </div>
        )}
      </div>
      <aside className="inspector">
        <h3>Run</h3>

        <div className="field">
          <label>Backend</label>
          <select
            value={backend}
            onChange={(e) => setBackend(e.target.value)}
            disabled={connected}
          >
            <option value="mock">mock</option>
            <option value="stepper">stepper</option>
            <option value="odrive">odrive</option>
            <option value="multiboard">multiboard</option>
          </select>
        </div>

        {needsDevice && (
          <div className="field">
            <label>Device (path or host:port)</label>
            <input
              type="text"
              value={device}
              onChange={(e) => setDevice(e.target.value)}
              disabled={connected}
              placeholder="/dev/ttyUSB0 or 127.0.0.1:5555"
            />
          </div>
        )}

        {needsBaud && (
          <div className="field">
            <label>Baud rate</label>
            <input
              type="number"
              value={baud}
              onChange={(e) => setBaud(parseInt(e.target.value, 10) || 115200)}
              disabled={connected}
            />
          </div>
        )}

        {needsAxisMap && (
          <div className="field">
            <label>axis_map JSON</label>
            <textarea
              value={axisMapText}
              onChange={(e) => setAxisMapText(e.target.value)}
              disabled={connected}
              rows={6}
              style={{
                background: "var(--bg-deep)",
                border: "1px solid var(--border)",
                color: "var(--text)",
                fontFamily: "var(--font-mono)",
                fontSize: "0.72rem",
                padding: "0.4rem",
                borderRadius: 4,
                resize: "vertical",
              }}
            />
          </div>
        )}

        <button
          type="button"
          className="btn"
          onClick={() => void (connected ? disconnect() : connect())}
        >
          {connected ? "Disconnect" : "Connect"}
        </button>

        <button
          type="button"
          className="btn"
          disabled={!connected || estopped}
          onClick={() => void home()}
        >
          Home
        </button>
        <button
          type="button"
          className="btn"
          disabled={!connected || estopped}
          onClick={() => void playLine()}
        >
          Play line
        </button>

        <label className="checkbox-row">
          <input
            type="checkbox"
            checked={closedLoop}
            onChange={(e) => setClosedLoop(e.target.checked)}
            disabled={!connected}
          />
          Closed-loop
        </label>
        <label className="checkbox-row">
          <input
            type="checkbox"
            checked={realtime}
            onChange={(e) => setRealtime(e.target.checked)}
            disabled={!connected}
          />
          Realtime
        </label>

        {connected && (
          <>
            <button
              type="button"
              className={`btn btn-danger ${estopped ? "danger" : ""}`}
              onClick={() => void triggerEstop()}
            >
              E-STOP
            </button>
            {estopped && (
              <button type="button" className="btn" onClick={() => void clearEstop()}>
                Clear E-stop
              </button>
            )}
          </>
        )}

        <div className="readout">{statusText}</div>
        {playReadout && <div className="readout pre">{playReadout}</div>}

        <div
          style={{
            border: "1px solid var(--border)",
            borderRadius: 6,
            padding: "0.5rem",
          }}
        >
          <strong style={{ fontSize: "0.75rem", color: "var(--accent)" }}>
            Safety limits
          </strong>
          <div className="readout pre" style={{ marginTop: "0.35rem" }}>
            {safetyText}
          </div>
        </div>

        <div className="readout" style={{ fontSize: "0.68rem" }}>
          Trajectory: [{traj.start.join(", ")}] → [{traj.end.join(", ")}] · {traj.segments}{" "}
          segments
          <br />
          Configure start/end on Simulate tab.
        </div>

        {error && (
          <div className="readout" style={{ color: "var(--danger)" }}>
            {error}
          </div>
        )}
      </aside>
    </div>
  );
}
