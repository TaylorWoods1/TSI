import { useEffect, useState } from "react";
import * as api from "../api/client";
import { useApp } from "../context";
import RobotScene, { type SceneData } from "../scene/RobotScene";

export default function RunPage() {
  const { venue, dolly, setDolly, traj } = useApp();
  const [backend, setBackend] = useState("mock");
  const [connected, setConnected] = useState(false);
  const [estopped, setEstopped] = useState(false);
  const [statusText, setStatusText] = useState("disconnected");
  const [closedLoop, setClosedLoop] = useState(false);
  const [realtime, setRealtime] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!connected) return;
    const id = setInterval(async () => {
      try {
        const s = await api.runStatus();
        setConnected(s.connected);
        setEstopped(s.estopped);
        if (s.pose) setDolly({ x: s.pose[0], y: s.pose[1], z: s.pose[2] });
        const steps = s.steps?.join(", ") ?? "—";
        setStatusText(
          `${s.backend ?? "—"} | estop: ${s.estopped} | steps: [${steps}]`,
        );
      } catch (e) {
        setError(String(e));
      }
    }, 200);
    return () => clearInterval(id);
  }, [connected, setDolly]);

  const connect = async () => {
    try {
      setError(null);
      await api.runConnect(backend);
      setConnected(true);
      setEstopped(false);
    } catch (e) {
      setError(String(e));
    }
  };

  const disconnect = async () => {
    await api.runDisconnect();
    setConnected(false);
    setStatusText("disconnected");
  };

  const playLine = async () => {
    try {
      setError(null);
      await api.runPlayLine({
        start: traj.start,
        end: traj.end,
        segments: traj.segments,
        closed_loop: closedLoop,
        realtime: realtime,
      });
    } catch (e) {
      setError(String(e));
    }
  };

  const scene: SceneData = {
    anchors: venue?.anchors ?? [],
    dolly,
    lengths: [],
  };

  return (
    <div className="page">
      <div className="viewport">
        <RobotScene scene={scene} />
      </div>
      <aside className="inspector">
        <h3>Run</h3>
        <div className="field">
          <label>Backend</label>
          <select value={backend} onChange={(e) => setBackend(e.target.value)} disabled={connected}>
            <option value="mock">mock</option>
            <option value="stepper" disabled>
              stepper (soon)
            </option>
          </select>
        </div>
        <button type="button" className="btn" onClick={connected ? disconnect : connect}>
          {connected ? "Disconnect" : "Connect"}
        </button>
        <button type="button" className="btn" disabled={!connected || estopped} onClick={() => api.runHome()}>
          Home
        </button>
        <button type="button" className="btn" disabled={!connected || estopped} onClick={playLine}>
          Play line
        </button>
        <label className="field">
          <input
            type="checkbox"
            checked={closedLoop}
            onChange={(e) => setClosedLoop(e.target.checked)}
          />{" "}
          Closed-loop
        </label>
        <label className="field">
          <input
            type="checkbox"
            checked={realtime}
            onChange={(e) => setRealtime(e.target.checked)}
          />{" "}
          Realtime
        </label>
        {connected && (
          <>
            <button type="button" className="btn btn-danger" onClick={() => api.runEstop().then(() => setEstopped(true))}>
              E-STOP
            </button>
            {estopped && (
              <button type="button" className="btn" onClick={() => api.runClearEstop().then(() => setEstopped(false))}>
                Clear E-stop
              </button>
            )}
          </>
        )}
        <div className="readout">{statusText}</div>
        {error && <div className="readout" style={{ color: "var(--danger)" }}>{error}</div>}
      </aside>
    </div>
  );
}
