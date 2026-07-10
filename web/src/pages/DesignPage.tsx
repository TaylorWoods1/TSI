import { useCallback, useEffect, useRef, useState } from "react";
import * as api from "../api/client";
import { useApp } from "../context";
import RobotScene, { type SceneData } from "../scene/RobotScene";

export default function DesignPage() {
  const { venue, setVenue, classify, setClassify, dolly, setDolly } = useApp();
  const [error, setError] = useState<string | null>(null);
  const [scene, setScene] = useState<SceneData | null>(null);
  const fileRef = useRef<HTMLInputElement>(null);

  const refreshScene = useCallback(async (pos: api.Vec3) => {
    try {
      const snap = await api.sceneSnapshot([pos.x, pos.y, pos.z]);
      const data = api.sceneSnapshotToSceneData(snap);
      setScene({
        ...data,
        show_pulls: venue?.model === "sag",
      });
    } catch {
      setScene({
        anchors: venue?.anchors ?? [],
        dolly: pos,
        lengths: [],
      });
    }
  }, [venue]);

  const loadPreset = useCallback(async () => {
    try {
      setError(null);
      const res = await api.fromPreset({
        kind: "rect",
        width: 10,
        depth: 6,
        height: 8,
        point_mass: true,
      });
      setVenue(res.venue);
      setClassify(res.classify);
      setDolly(res.venue.home);
    } catch (e) {
      setError(String(e));
    }
  }, [setVenue, setClassify, setDolly]);

  useEffect(() => {
    if (!venue) loadPreset();
  }, [venue, loadPreset]);

  useEffect(() => {
    if (venue) refreshScene(dolly);
  }, [venue, dolly, refreshScene]);

  const pushAnchors = async (anchors: api.Vec3[]) => {
    if (!venue) return;
    try {
      setError(null);
      const res = await api.setAnchors({
        anchors,
        point_mass: venue.point_mass,
        model: venue.model,
        pulley_radius: venue.pulley_radius,
        sag_mu: venue.sag_mu,
        sag_ea: venue.sag_ea,
      });
      setVenue(res.venue);
      setClassify(res.classify);
    } catch (e) {
      setError(String(e));
    }
  };

  const applyModel = async (patch: Partial<api.Venue>) => {
    if (!venue) return;
    try {
      setError(null);
      const res = await api.setCableModel({
        model: patch.model ?? venue.model,
        pulley_radius: patch.pulley_radius ?? venue.pulley_radius,
        sag_mu: patch.sag_mu ?? venue.sag_mu,
        sag_ea: patch.sag_ea ?? venue.sag_ea,
      });
      setVenue(res.venue);
      setClassify(res.classify);
    } catch (e) {
      setError(String(e));
    }
  };

  const onAnchorDrag = (index: number, pos: api.Vec3) => {
    if (!venue) return;
    const next = venue.anchors.map((a, i) => (i === index ? pos : a));
    setVenue({ ...venue, anchors: next });
  };

  const onAnchorBlur = () => {
    if (venue) pushAnchors(venue.anchors);
  };

  const updateAnchorField = (index: number, axis: "x" | "y" | "z", value: number) => {
    if (!venue) return;
    const next = venue.anchors.map((a, i) =>
      i === index ? { ...a, [axis]: value } : a,
    );
    setVenue({ ...venue, anchors: next });
  };

  const saveToml = async () => {
    try {
      const { toml } = await api.getToml();
      const blob = new Blob([toml], { type: "text/plain" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "venue.toml";
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      setError(String(e));
    }
  };

  const loadFile = async (file: File) => {
    try {
      const text = await file.text();
      const res = await api.loadVenue(text);
      setVenue(res.venue);
      setClassify(res.classify);
      setDolly(res.venue.home);
    } catch (e) {
      setError(String(e));
    }
  };

  const displayScene: SceneData = scene ?? {
    anchors: venue?.anchors ?? [],
    dolly,
    lengths: [],
    model: venue?.model,
  };

  return (
    <div className="page">
      <div className="viewport">
        <RobotScene
          scene={displayScene}
          draggable
          onAnchorDrag={onAnchorDrag}
        />
      </div>
      <aside className="inspector">
        <h3>Design</h3>
        <button type="button" className="btn" onClick={loadPreset}>
          Rect 4 preset
        </button>
        <button type="button" className="btn" onClick={saveToml}>
          Save TOML
        </button>
        <button type="button" className="btn" onClick={() => fileRef.current?.click()}>
          Load TOML
        </button>
        <input
          ref={fileRef}
          className="hidden-input"
          type="file"
          accept=".toml,text/plain"
          onChange={(e) => {
            const f = e.target.files?.[0];
            if (f) loadFile(f);
            e.target.value = "";
          }}
        />
        <div className="field">
          <label>Cable model</label>
          <select
            value={venue?.model ?? "ideal"}
            onChange={(e) => applyModel({ model: e.target.value })}
          >
            <option value="ideal">Ideal (straight)</option>
            <option value="pulley">Pulley (tangent + wrap)</option>
            <option value="sag">Sag (catenary)</option>
          </select>
        </div>
        {venue?.model === "pulley" && (
          <div className="field">
            <label>Pulley radius (m)</label>
            <input
              type="number"
              step="0.01"
              min={0.01}
              value={venue.pulley_radius}
              onChange={(e) =>
                setVenue({ ...venue, pulley_radius: parseFloat(e.target.value) || 0.05 })
              }
              onBlur={() => applyModel({ pulley_radius: venue.pulley_radius })}
            />
          </div>
        )}
        {venue?.model === "sag" && (
          <>
            <div className="field">
              <label>Sag μ (kg/m)</label>
              <input
                type="number"
                step="0.1"
                value={venue.sag_mu}
                onChange={(e) =>
                  setVenue({ ...venue, sag_mu: parseFloat(e.target.value) || 1 })
                }
                onBlur={() => applyModel({ sag_mu: venue.sag_mu })}
              />
            </div>
            <div className="field">
              <label>Sag EA (N)</label>
              <input
                type="number"
                step="10000"
                value={venue.sag_ea}
                onChange={(e) =>
                  setVenue({ ...venue, sag_ea: parseFloat(e.target.value) || 1e6 })
                }
                onBlur={() => applyModel({ sag_ea: venue.sag_ea })}
              />
            </div>
          </>
        )}
        <div className="readout">Classify: {classify}</div>
        <div className="readout">Model: {venue?.model ?? "—"}</div>
        {error && <div className="readout" style={{ color: "var(--danger)" }}>{error}</div>}
        <div className="anchor-list">
          {venue?.anchors.map((a, i) => (
            <div key={i} className="anchor-card" onBlur={onAnchorBlur}>
              <strong>Anchor {i + 1}</strong>
              <div className="field-row">
                {(["x", "y", "z"] as const).map((axis) => (
                  <div key={axis} className="field">
                    <label>{axis}</label>
                    <input
                      type="number"
                      step="0.1"
                      value={a[axis]}
                      onChange={(e) =>
                        updateAnchorField(i, axis, parseFloat(e.target.value) || 0)
                      }
                    />
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      </aside>
    </div>
  );
}
