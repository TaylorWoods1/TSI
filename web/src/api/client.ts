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

export type Venue = {
  anchors: Vec3[];
  attachments: Vec3[];
  point_mass: boolean;
  model: string;
  home: Vec3;
};

export type VenueResponse = { venue: Venue; classify: string };

export async function health() {
  return request<{ ok: boolean; version: string }>("/health");
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

export async function ik(xyz: [number, number, number], mg?: number) {
  return request<{ lengths: number[]; tensions?: number[] }>("/ik", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ xyz, mg }),
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

export async function sceneSnapshot(xyz: [number, number, number]) {
  return request<{
    anchors: [number, number, number][];
    dolly: [number, number, number];
    attachments: [number, number, number][];
    lengths: number[];
  }>("/scene/snapshot", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ xyz }),
  });
}

export async function runConnect(backend: string) {
  return request<{ ok: boolean; axes: number }>("/run/connect", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ backend }),
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
  }>("/run/status");
}
