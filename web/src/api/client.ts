const BASE =
  import.meta.env.DEV &&
  typeof window !== "undefined" &&
  window.location.port === "5173"
    ? ""
    : "http://127.0.0.1:7700";

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const r = await fetch(`${BASE}${path}`, init);
  if (!r.ok) {
    const text = await r.text();
    try {
      const j = JSON.parse(text) as { error?: string };
      throw new Error(j.error ?? text);
    } catch {
      throw new Error(text);
    }
  }
  return r.json() as Promise<T>;
}

export type Vec3 = { x: number; y: number; z: number };

export type Anchor = Vec3 & {
  pulley_axis?: Vec3 | null;
  pulley_radius?: number;
  pulley_winch_exit?: Vec3 | null;
  pulley_runout_m?: number;
};

export type Venue = {
  anchors: Anchor[];
  attachments: Vec3[];
  point_mass: boolean;
  model: string;
  pulley_radius: number;
  sag_mu: number;
  sag_ea: number;
  home: Vec3;
};

export type VenueResponse = { venue: Venue; classify: string };

export type SceneSnapshot = {
  anchors: [number, number, number][];
  dolly: [number, number, number];
  attachments: [number, number, number][];
  lengths: number[];
  cable_paths: [number, number, number][][];
  unit_pulls: [number, number, number][];
  model: string;
};

function vec3FromTuple(t: [number, number, number]): Vec3 {
  return { x: t[0], y: t[1], z: t[2] };
}

export function sceneSnapshotToSceneData(snap: SceneSnapshot): {
  anchors: Vec3[];
  dolly: Vec3;
  attachments: Vec3[];
  lengths: number[];
  cable_paths: Vec3[][];
  unit_pulls: Vec3[];
  model: string;
} {
  return {
    anchors: snap.anchors.map(vec3FromTuple),
    dolly: vec3FromTuple(snap.dolly),
    attachments: snap.attachments.map(vec3FromTuple),
    lengths: snap.lengths,
    cable_paths: snap.cable_paths.map((path) => path.map(vec3FromTuple)),
    unit_pulls: snap.unit_pulls.map(vec3FromTuple),
    model: snap.model,
  };
}

export async function health() {
  return request<{ ok: boolean; version: string }>("/health");
}

export async function getVenue() {
  return request<VenueResponse>("/venue");
}

export async function fromPreset(body: object) {
  return request<VenueResponse>("/venue/from_preset", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
}

export async function setAnchors(body: object) {
  return request<VenueResponse>("/venue/set_anchors", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
}

export async function setHome(home: [number, number, number]) {
  return request<VenueResponse>("/venue/home", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ home }),
  });
}

export async function setCableModel(body: {
  model: string;
  pulley_radius?: number;
  sag_mu?: number;
  sag_ea?: number;
}) {
  return request<VenueResponse>("/venue/set_model", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
}

export async function loadVenue(toml: string) {
  return request<VenueResponse>("/venue/load", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ toml }),
  });
}

export async function getToml() {
  return request<{ toml: string }>("/venue/toml");
}

export async function ik(
  xyz: [number, number, number],
  options?: { mg?: number; model?: string },
) {
  return request<{
    lengths: number[];
    tensions?: number[];
    unstrained_lengths?: (number | null)[];
  }>("/ik", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ xyz, mg: options?.mg, model: options?.model }),
  });
}

export async function fk(
  lengths: number[],
  seed: [number, number, number],
  options?: {
    orientation_rv?: [number, number, number];
    tensions?: number[];
    allow_underconstrained?: boolean;
  },
) {
  return request<{
    xyz: [number, number, number];
    orientation_rv: [number, number, number];
    method: string;
    residual: number;
  }>("/fk", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ lengths, seed, ...options }),
  });
}

export async function jacobian(
  xyz: [number, number, number],
  orientation_rv?: [number, number, number],
) {
  return request<{ rows: number[][]; cols: number }>("/jacobian", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ xyz, orientation_rv }),
  });
}

export async function feasible(
  xyz: [number, number, number],
  opts?: { mg?: number; f_min?: number; f_max?: number },
) {
  return request<{ ok: boolean }>("/feasible", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ xyz, ...opts }),
  });
}

export async function trajLine(body: object) {
  return request<{ waypoints: [number, number, number][]; lengths: number[][] }>(
    "/traj/line",
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    },
  );
}

export async function trajWaypoints(waypoints: [number, number, number][]) {
  return request<{ waypoints: [number, number, number][]; lengths: number[][] }>(
    "/traj/waypoints",
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ waypoints }),
    },
  );
}

export async function workspace(body: object) {
  return request<{
    fraction: number;
    samples: { x: number; y: number; z: number; feasible: boolean }[];
  }>("/workspace", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
}

export async function sceneSnapshot(
  xyz: [number, number, number],
  orientation_rv?: [number, number, number],
) {
  return request<SceneSnapshot>("/scene/snapshot", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ xyz, orientation_rv }),
  });
}

export async function sceneExport(body: object) {
  return request<{ html: string }>("/scene/export", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
}

export async function runConnect(body: {
  backend: string;
  device?: string;
  baud?: number;
  axis_map?: object;
}) {
  return request<{ ok: boolean; axes: number }>("/run/connect", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
}

export async function runDisconnect() {
  return request<{ ok: boolean }>("/run/disconnect", { method: "POST" });
}

export async function runHome() {
  return request<{ ok: boolean }>("/run/home", { method: "POST" });
}

export async function runPlayLine(body: object) {
  return request<{ final_steps: number[]; feedback_pose?: [number, number, number] }>(
    "/run/play_line",
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    },
  );
}

export async function runEstop() {
  return request<{ ok: boolean }>("/run/estop", { method: "POST" });
}

export async function runClearEstop() {
  return request<{ ok: boolean }>("/run/clear_estop", { method: "POST" });
}

export async function runStatus() {
  return request<{
    connected: boolean;
    backend?: string;
    estopped: boolean;
    steps?: number[];
    pose?: [number, number, number];
    safety?: {
      min: [number, number, number];
      max: [number, number, number];
      max_speed_mps: number;
    };
  }>("/run/status");
}

export type Calibration = {
  home: [number, number, number];
  home_lengths_m: number[];
  drum_radius_m: number;
  steps_per_rev: number;
  anchors_m?: [number, number, number][] | null;
  saved_at: string;
};

export async function getCalibration() {
  return request<Calibration>("/calibration");
}

export async function captureCalibration(body: {
  home?: [number, number, number];
  drum_radius_m: number;
  steps_per_rev: number;
}) {
  return request<Calibration>("/calibration/capture", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
}

export async function setCalibrationAnchor(index: number, exit: [number, number, number]) {
  return request<Calibration>("/calibration/anchor", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ index, exit }),
  });
}

export async function applyCalibration() {
  return request<VenueResponse>("/calibration/apply", { method: "POST" });
}

export async function getCalibrationJson() {
  return request<{ json: string }>("/calibration/json");
}

export async function loadCalibration(json: string) {
  return request<Calibration>("/calibration/load", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ json }),
  });
}
