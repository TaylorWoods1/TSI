import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";
import * as api from "../api/client";
import { useApp } from "../context";
import RobotScene, { type SceneData } from "../scene/RobotScene";

const AXIS_PRESETS: Record<string, api.Vec3> = {
  Z: { x: 0, y: 0, z: 1 },
  X: { x: 1, y: 0, z: 0 },
  Y: { x: 0, y: 1, z: 0 },
};

function axisPresetKey(axis?: api.Vec3 | null): string {
  if (!axis) return "Z";
  for (const [k, v] of Object.entries(AXIS_PRESETS)) {
    if (
      Math.abs(axis.x - v.x) < 1e-6 &&
      Math.abs(axis.y - v.y) < 1e-6 &&
      Math.abs(axis.z - v.z) < 1e-6
    ) {
      return k;
    }
  }
  return "custom";
}

function Collapsible({
  title,
  children,
  defaultOpen = false,
}: {
  title: string;
  children: ReactNode;
  defaultOpen?: boolean;
}) {
  return (
    <details
      open={defaultOpen}
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
        }}
      >
        {title}
      </summary>
      <div style={{ marginTop: "0.5rem", display: "flex", flexDirection: "column", gap: "0.5rem" }}>
        {children}
      </div>
    </details>
  );
}

export default function DesignPage() {
  const { venue, setVenue, classify, setClassify, dolly, setDolly, scene, sceneLoading } =
    useApp();
  const [error, setError] = useState<string | null>(null);
  const [presetKind, setPresetKind] = useState<"rect" | "polygon">("rect");
  const [rectW, setRectW] = useState(10);
  const [rectD, setRectD] = useState(6);
  const [rectH, setRectH] = useState(8);
  const [polyN, setPolyN] = useState(6);
  const [polyR, setPolyR] = useState(5);
  const [polyH, setPolyH] = useState(8);
  const [presetPointMass, setPresetPointMass] = useState(true);
  const [homeDraft, setHomeDraft] = useState<api.Vec3>({ x: 0, y: 0, z: 2 });
  const [drumRadius, setDrumRadius] = useState(0.05);
  const [stepsPerRev, setStepsPerRev] = useState(800);
  const [calAnchors, setCalAnchors] = useState<[number, number, number][]>([]);
  const [motorAxes, setMotorAxes] = useState<api.MotorAxis[]>([]);
  const fileRef = useRef<HTMLInputElement>(null);
  const calFileRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (venue) setHomeDraft({ ...venue.home });
  }, [venue?.home.x, venue?.home.y, venue?.home.z]);

  useEffect(() => {
    api.getMotors()
      .then((res) => setMotorAxes(res.axes))
      .catch(() => {});
  }, [venue?.anchors.length]);

  useEffect(() => {
    api.getCalibration()
      .then((cal) => {
        setDrumRadius(cal.drum_radius_m);
        setStepsPerRev(cal.steps_per_rev);
        if (cal.anchors_m?.length) {
          setCalAnchors(cal.anchors_m);
        } else if (venue) {
          setCalAnchors(venue.anchors.map((a) => [a.x, a.y, a.z]));
        }
      })
      .catch(() => {});
  }, [venue]);

  const applyVenueResponse = (res: api.VenueResponse) => {
    setVenue(res.venue);
    setClassify(res.classify);
    setHomeDraft({ ...res.venue.home });
  };

  const pushAnchors = useCallback(
    async (nextVenue: api.Venue) => {
      try {
        setError(null);
        const res = await api.setAnchors({
          anchors: nextVenue.anchors,
          attachments: nextVenue.point_mass
            ? nextVenue.attachments.map(() => ({ x: 0, y: 0, z: 0 }))
            : nextVenue.attachments,
          point_mass: nextVenue.point_mass,
          model: nextVenue.model,
          pulley_radius: nextVenue.pulley_radius,
          sag_mu: nextVenue.sag_mu,
          sag_ea: nextVenue.sag_ea,
        });
        applyVenueResponse(res);
      } catch (e) {
        setError(String(e));
      }
    },
    [setVenue, setClassify],
  );

  const applyPreset = async () => {
    try {
      setError(null);
      const body =
        presetKind === "rect"
          ? {
              kind: "rect",
              width: rectW,
              depth: rectD,
              height: rectH,
              point_mass: presetPointMass,
            }
          : {
              kind: "polygon",
              n: polyN,
              radius: polyR,
              height: polyH,
              point_mass: presetPointMass,
            };
      const res = await api.fromPreset(body);
      applyVenueResponse(res);
      setDolly(res.venue.home);
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
      applyVenueResponse(res);
    } catch (e) {
      setError(String(e));
    }
  };

  const setHomePose = async (home: api.Vec3) => {
    try {
      setError(null);
      const res = await api.setHome([home.x, home.y, home.z]);
      applyVenueResponse(res);
    } catch (e) {
      setError(String(e));
    }
  };

  const onAnchorDrag = (index: number, pos: api.Vec3) => {
    if (!venue) return;
    const next = venue.anchors.map((a, i) => (i === index ? { ...a, ...pos } : a));
    setVenue({ ...venue, anchors: next });
  };

  const onAnchorBlur = () => {
    if (venue) void pushAnchors(venue);
  };

  const updateAnchorField = (
    index: number,
    patch: Partial<api.Anchor>,
  ) => {
    if (!venue) return;
    const next = venue.anchors.map((a, i) => (i === index ? { ...a, ...patch } : a));
    setVenue({ ...venue, anchors: next });
  };

  const updateAttachment = (index: number, axis: "x" | "y" | "z", value: number) => {
    if (!venue) return;
    const attachments = [...venue.attachments];
    while (attachments.length < venue.anchors.length) {
      attachments.push({ x: 0, y: 0, z: 0 });
    }
    attachments[index] = { ...attachments[index], [axis]: value };
    setVenue({ ...venue, attachments });
  };

  const togglePointMass = (checked: boolean) => {
    if (!venue) return;
    const attachments = checked
      ? venue.attachments.map(() => ({ x: 0, y: 0, z: 0 }))
      : venue.attachments.length === venue.anchors.length
        ? venue.attachments
        : venue.anchors.map(() => ({ x: 0, y: 0, z: 0 }));
    const next = { ...venue, point_mass: checked, attachments };
    setVenue(next);
    void pushAnchors(next);
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
      applyVenueResponse(res);
      setDolly(res.venue.home);
    } catch (e) {
      setError(String(e));
    }
  };

  const saveMotorMapping = async () => {
    if (!venue) return;
    try {
      setError(null);
      const axes = venue.anchors.map((_, i) => ({
        drum_radius_m: motorAxes[i]?.drum_radius_m ?? 0.05,
        steps_per_rev: motorAxes[i]?.steps_per_rev ?? 200,
      }));
      const res = await api.setMotors(axes);
      setMotorAxes(res.axes);
    } catch (e) {
      setError(String(e));
    }
  };

  const captureCalibration = async () => {
    try {
      setError(null);
      const cal = await api.captureCalibration({
        home: [homeDraft.x, homeDraft.y, homeDraft.z],
        drum_radius_m: drumRadius,
        steps_per_rev: stepsPerRev,
      });
      setDrumRadius(cal.drum_radius_m);
      setStepsPerRev(cal.steps_per_rev);
      if (cal.anchors_m) setCalAnchors(cal.anchors_m);
    } catch (e) {
      setError(String(e));
    }
  };

  const measureAnchor = async (index: number) => {
    const exit = calAnchors[index];
    if (!exit) return;
    try {
      setError(null);
      const cal = await api.setCalibrationAnchor(index, exit);
      if (cal.anchors_m) setCalAnchors(cal.anchors_m);
    } catch (e) {
      setError(String(e));
    }
  };

  const exportCalibrationJson = async () => {
    try {
      const { json } = await api.getCalibrationJson();
      const blob = new Blob([json], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "calibration.json";
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      setError(String(e));
    }
  };

  const exportCalibrationVenueToml = async () => {
    try {
      setError(null);
      const { toml } = await api.getCalibrationVenueToml();
      const blob = new Blob([toml], { type: "text/plain" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "venue_from_cal.toml";
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      setError(String(e));
    }
  };

  const applyCalibration = async () => {
    try {
      setError(null);
      const res = await api.applyCalibration();
      applyVenueResponse(res);
      setDolly(res.venue.home);
    } catch (e) {
      setError(String(e));
    }
  };

  const loadCalibrationFile = async (file: File) => {
    try {
      const json = await file.text();
      const cal = await api.loadCalibration(json);
      setDrumRadius(cal.drum_radius_m);
      setStepsPerRev(cal.steps_per_rev);
      if (cal.anchors_m) setCalAnchors(cal.anchors_m);
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

  return (
    <div className="page">
      <div className="viewport">
        <RobotScene scene={displayScene} draggable onAnchorDrag={onAnchorDrag} />
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
        <h3>Design</h3>

        <Collapsible title="Layout preset" defaultOpen>
          <div className="field">
            <label>Preset type</label>
            <select
              value={presetKind}
              onChange={(e) => setPresetKind(e.target.value as "rect" | "polygon")}
            >
              <option value="rect">Rectangle</option>
              <option value="polygon">Polygon</option>
            </select>
          </div>
          {presetKind === "rect" ? (
            <div className="field-row">
              <div className="field">
                <label>Width (m)</label>
                <input
                  type="number"
                  step="0.5"
                  value={rectW}
                  onChange={(e) => setRectW(parseFloat(e.target.value) || 1)}
                />
              </div>
              <div className="field">
                <label>Depth (m)</label>
                <input
                  type="number"
                  step="0.5"
                  value={rectD}
                  onChange={(e) => setRectD(parseFloat(e.target.value) || 1)}
                />
              </div>
              <div className="field">
                <label>Height (m)</label>
                <input
                  type="number"
                  step="0.5"
                  value={rectH}
                  onChange={(e) => setRectH(parseFloat(e.target.value) || 1)}
                />
              </div>
            </div>
          ) : (
            <div className="field-row">
              <div className="field">
                <label>N cables</label>
                <input
                  type="number"
                  min={3}
                  value={polyN}
                  onChange={(e) => setPolyN(parseInt(e.target.value, 10) || 3)}
                />
              </div>
              <div className="field">
                <label>Radius (m)</label>
                <input
                  type="number"
                  step="0.5"
                  value={polyR}
                  onChange={(e) => setPolyR(parseFloat(e.target.value) || 1)}
                />
              </div>
              <div className="field">
                <label>Height (m)</label>
                <input
                  type="number"
                  step="0.5"
                  value={polyH}
                  onChange={(e) => setPolyH(parseFloat(e.target.value) || 1)}
                />
              </div>
            </div>
          )}
          <label className="checkbox-row">
            <input
              type="checkbox"
              checked={presetPointMass}
              onChange={(e) => setPresetPointMass(e.target.checked)}
            />
            Point mass
          </label>
          <button type="button" className="btn" onClick={() => void applyPreset()}>
            Apply preset
          </button>
        </Collapsible>

        <label className="checkbox-row">
          <input
            type="checkbox"
            checked={venue?.point_mass ?? true}
            onChange={(e) => togglePointMass(e.target.checked)}
          />
          Point mass (cables meet at dolly origin)
        </label>

        {!venue?.point_mass && venue && (
          <Collapsible title="Attachments (body frame)" defaultOpen>
            {venue.anchors.map((_, i) => {
              const att = venue.attachments[i] ?? { x: 0, y: 0, z: 0 };
              return (
                <div key={i} className="anchor-card" onBlur={onAnchorBlur}>
                  <strong>Cable {i + 1}</strong>
                  <div className="field-row">
                    {(["x", "y", "z"] as const).map((axis) => (
                      <div key={axis} className="field">
                        <label>{axis}</label>
                        <input
                          type="number"
                          step="0.01"
                          value={att[axis]}
                          onChange={(e) =>
                            updateAttachment(i, axis, parseFloat(e.target.value) || 0)
                          }
                        />
                      </div>
                    ))}
                  </div>
                </div>
              );
            })}
          </Collapsible>
        )}

        <Collapsible title="Home pose">
          <div className="field-row">
            {(["x", "y", "z"] as const).map((axis) => (
              <div key={axis} className="field">
                <label>{axis}</label>
                <input
                  type="number"
                  step="0.1"
                  value={homeDraft[axis]}
                  onChange={(e) =>
                    setHomeDraft({ ...homeDraft, [axis]: parseFloat(e.target.value) || 0 })
                  }
                  onBlur={() => void setHomePose(homeDraft)}
                />
              </div>
            ))}
          </div>
          <button
            type="button"
            className="btn"
            onClick={() => {
              const home = { ...dolly };
              setHomeDraft(home);
              void setHomePose(home);
            }}
          >
            Set home from dolly
          </button>
        </Collapsible>

        <div className="field">
          <label>Cable model</label>
          <select
            value={venue?.model ?? "ideal"}
            onChange={(e) => void applyModel({ model: e.target.value })}
          >
            <option value="ideal">Ideal (straight)</option>
            <option value="pulley">Pulley (tangent + wrap)</option>
            <option value="sag">Sag (catenary)</option>
          </select>
        </div>
        {venue?.model === "pulley" && (
          <div className="field">
            <label>Default pulley radius (m)</label>
            <input
              type="number"
              step="0.01"
              min={0.01}
              value={venue.pulley_radius}
              onChange={(e) =>
                setVenue({
                  ...venue,
                  pulley_radius: parseFloat(e.target.value) || 0.05,
                })
              }
              onBlur={() => void applyModel({ pulley_radius: venue.pulley_radius })}
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
                onBlur={() => void applyModel({ sag_mu: venue.sag_mu })}
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
                onBlur={() => void applyModel({ sag_ea: venue.sag_ea })}
              />
            </div>
          </>
        )}

        <div className="readout">Classify: {classify}</div>

        <div className="anchor-list">
          {venue?.anchors.map((a, i) => {
            const axisKey = axisPresetKey(a.pulley_axis);
            const winch = a.pulley_winch_exit ?? { x: 1, y: 0, z: 0 };
            return (
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
                          updateAnchorField(i, { [axis]: parseFloat(e.target.value) || 0 })
                        }
                      />
                    </div>
                  ))}
                </div>
                {venue.model === "pulley" && (
                  <>
                    <div className="field">
                      <label>Pulley radius override (0 = default)</label>
                      <input
                        type="number"
                        step="0.01"
                        min={0}
                        value={a.pulley_radius ?? 0}
                        onChange={(e) =>
                          updateAnchorField(i, {
                            pulley_radius: parseFloat(e.target.value) || 0,
                          })
                        }
                      />
                    </div>
                    <div className="field">
                      <label>Pulley axis</label>
                      <select
                        value={axisKey}
                        onChange={(e) => {
                          const v = e.target.value;
                          if (v !== "custom") {
                            updateAnchorField(i, { pulley_axis: AXIS_PRESETS[v] });
                          }
                        }}
                      >
                        <option value="Z">Z-up</option>
                        <option value="X">X</option>
                        <option value="Y">Y</option>
                        <option value="custom">Custom</option>
                      </select>
                    </div>
                    {axisKey === "custom" && (
                      <div className="field-row">
                        {(["x", "y", "z"] as const).map((axis) => (
                          <div key={axis} className="field">
                            <label>axis {axis}</label>
                            <input
                              type="number"
                              step="0.1"
                              value={a.pulley_axis?.[axis] ?? 0}
                              onChange={(e) => {
                                const cur = a.pulley_axis ?? { x: 0, y: 0, z: 1 };
                                updateAnchorField(i, {
                                  pulley_axis: {
                                    ...cur,
                                    [axis]: parseFloat(e.target.value) || 0,
                                  },
                                });
                              }}
                            />
                          </div>
                        ))}
                      </div>
                    )}
                    <Collapsible title="Advanced pulley">
                      <div className="field">
                        <label>Winch exit (unit vector)</label>
                        <div className="field-row">
                          {(["x", "y", "z"] as const).map((axis) => (
                            <div key={axis} className="field">
                              <label>{axis}</label>
                              <input
                                type="number"
                                step="0.1"
                                value={winch[axis]}
                                onChange={(e) =>
                                  updateAnchorField(i, {
                                    pulley_winch_exit: {
                                      ...winch,
                                      [axis]: parseFloat(e.target.value) || 0,
                                    },
                                  })
                                }
                              />
                            </div>
                          ))}
                        </div>
                      </div>
                      <div className="field">
                        <label>Runout (m)</label>
                        <input
                          type="number"
                          step="0.01"
                          min={0}
                          value={a.pulley_runout_m ?? 0}
                          onChange={(e) =>
                            updateAnchorField(i, {
                              pulley_runout_m: parseFloat(e.target.value) || 0,
                            })
                          }
                        />
                      </div>
                    </Collapsible>
                  </>
                )}
              </div>
            );
          })}
        </div>

        <Collapsible title="Motor mapping (per cable)">
          {venue?.anchors.map((_, i) => {
            const m = motorAxes[i] ?? { drum_radius_m: 0.05, steps_per_rev: 200 };
            return (
              <div key={i} className="anchor-card">
                <strong>Cable {i + 1}</strong>
                <div className="field-row">
                  <div className="field">
                    <label>Drum radius (m)</label>
                    <input
                      type="number"
                      step="0.001"
                      min={0.001}
                      value={m.drum_radius_m}
                      onChange={(e) => {
                        const next = [...motorAxes];
                        while (next.length <= i) {
                          next.push({ drum_radius_m: 0.05, steps_per_rev: 200 });
                        }
                        next[i] = {
                          ...next[i],
                          drum_radius_m: parseFloat(e.target.value) || 0.05,
                        };
                        setMotorAxes(next);
                      }}
                    />
                  </div>
                  <div className="field">
                    <label>Steps / rev</label>
                    <input
                      type="number"
                      step="1"
                      min={1}
                      value={m.steps_per_rev}
                      onChange={(e) => {
                        const next = [...motorAxes];
                        while (next.length <= i) {
                          next.push({ drum_radius_m: 0.05, steps_per_rev: 200 });
                        }
                        next[i] = {
                          ...next[i],
                          steps_per_rev: parseInt(e.target.value, 10) || 200,
                        };
                        setMotorAxes(next);
                      }}
                    />
                  </div>
                </div>
              </div>
            );
          })}
          <button
            type="button"
            className="btn"
            onClick={() => void saveMotorMapping()}
          >
            Save motor mapping
          </button>
        </Collapsible>

        <Collapsible title="Field calibration">
          <div className="field-row">
            <div className="field">
              <label>Drum radius (m)</label>
              <input
                type="number"
                step="0.001"
                value={drumRadius}
                onChange={(e) => setDrumRadius(parseFloat(e.target.value) || 0.05)}
              />
            </div>
            <div className="field">
              <label>Steps / rev</label>
              <input
                type="number"
                step="1"
                value={stepsPerRev}
                onChange={(e) => setStepsPerRev(parseInt(e.target.value, 10) || 800)}
              />
            </div>
          </div>
          <button type="button" className="btn" onClick={() => void captureCalibration()}>
            Capture calibration
          </button>
          {venue?.anchors.map((_, i) => {
            const m = calAnchors[i] ?? [0, 0, 0];
            return (
              <div key={i} className="anchor-card">
                <strong>Measure anchor {i + 1}</strong>
                <div className="field-row">
                  {[0, 1, 2].map((j) => (
                    <div key={j} className="field">
                      <label>{["x", "y", "z"][j]}</label>
                      <input
                        type="number"
                        step="0.01"
                        value={m[j]}
                        onChange={(e) => {
                          const next = [...calAnchors];
                          while (next.length <= i) next.push([0, 0, 0]);
                          const row = [...(next[i] ?? [0, 0, 0])] as [number, number, number];
                          row[j] = parseFloat(e.target.value) || 0;
                          next[i] = row;
                          setCalAnchors(next);
                        }}
                      />
                    </div>
                  ))}
                </div>
                <button type="button" className="btn" onClick={() => void measureAnchor(i)}>
                  Set measured exit
                </button>
              </div>
            );
          })}
          <button type="button" className="btn" onClick={() => void exportCalibrationJson()}>
            Export JSON
          </button>
          <button type="button" className="btn" onClick={() => void exportCalibrationVenueToml()}>
            Export venue TOML
          </button>
          <button type="button" className="btn" onClick={() => calFileRef.current?.click()}>
            Load JSON
          </button>
          <input
            ref={calFileRef}
            className="hidden-input"
            type="file"
            accept=".json,application/json"
            onChange={(e) => {
              const f = e.target.files?.[0];
              if (f) void loadCalibrationFile(f);
              e.target.value = "";
            }}
          />
          <button type="button" className="btn" onClick={() => void applyCalibration()}>
            Apply to venue
          </button>
        </Collapsible>

        <button type="button" className="btn" onClick={() => void saveToml()}>
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
            if (f) void loadFile(f);
            e.target.value = "";
          }}
        />

        {error && (
          <div className="readout" style={{ color: "var(--danger)" }}>
            {error}
          </div>
        )}
      </aside>
    </div>
  );
}
