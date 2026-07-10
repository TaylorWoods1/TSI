import { createContext, useContext, useEffect, useState, type ReactNode } from "react";
import * as api from "./api/client";
import { useSceneSnapshot } from "./hooks/useSceneSnapshot";
import type { SceneData } from "./scene/RobotScene";

export type TrajConfig = {
  start: [number, number, number];
  end: [number, number, number];
  segments: number;
};

type AppContextValue = {
  venue: api.Venue | null;
  setVenue: (v: api.Venue | null) => void;
  classify: string;
  setClassify: (c: string) => void;
  dolly: api.Vec3;
  setDolly: (d: api.Vec3) => void;
  orientationRv: [number, number, number];
  setOrientationRv: (rv: [number, number, number]) => void;
  traj: TrajConfig;
  setTraj: (t: TrajConfig) => void;
  scene: SceneData | null;
  sceneLoading: boolean;
  refreshScene: (pos: api.Vec3, ori?: [number, number, number]) => Promise<void>;
  workspacePts: api.Vec3[];
  setWorkspacePts: (pts: api.Vec3[]) => void;
  showPulls: boolean;
  setShowPulls: (v: boolean) => void;
  fkResidual: number | null;
  setFkResidual: (r: number | null) => void;
  runBackend: string | null;
  setRunBackend: (b: string | null) => void;
};

const AppContext = createContext<AppContextValue | null>(null);

export function AppProvider({ children }: { children: ReactNode }) {
  const [venue, setVenue] = useState<api.Venue | null>(null);
  const [classify, setClassify] = useState("—");
  const [dolly, setDolly] = useState<api.Vec3>({ x: 0, y: 0, z: 2 });
  const [orientationRv, setOrientationRv] = useState<[number, number, number]>([0, 0, 0]);
  const [traj, setTraj] = useState<TrajConfig>({
    start: [0, 0, 2],
    end: [0.5, 0, 2],
    segments: 8,
  });
  const [workspacePts, setWorkspacePts] = useState<api.Vec3[]>([]);
  const [showPulls, setShowPulls] = useState(true);
  const [fkResidual, setFkResidual] = useState<number | null>(null);
  const [runBackend, setRunBackend] = useState<string | null>(null);

  const { scene, sceneLoading, refreshScene } = useSceneSnapshot(dolly, {
    orientationRv: venue && !venue.point_mass ? orientationRv : undefined,
    workspace: workspacePts,
    showPulls,
    enabled: !!venue,
  });

  useEffect(() => {
    api.getVenue().then((res) => {
      setVenue(res.venue);
      setClassify(res.classify);
      setDolly(res.venue.home);
    }).catch(() => {
      api.fromPreset({
        kind: "rect",
        width: 10,
        depth: 6,
        height: 8,
        point_mass: true,
      }).then((res) => {
        setVenue(res.venue);
        setClassify(res.classify);
        setDolly(res.venue.home);
      }).catch(() => {});
    });
  }, []);

  return (
    <AppContext.Provider
      value={{
        venue,
        setVenue,
        classify,
        setClassify,
        dolly,
        setDolly,
        orientationRv,
        setOrientationRv,
        traj,
        setTraj,
        scene,
        sceneLoading,
        refreshScene,
        workspacePts,
        setWorkspacePts,
        showPulls,
        setShowPulls,
        fkResidual,
        setFkResidual,
        runBackend,
        setRunBackend,
      }}
    >
      {children}
    </AppContext.Provider>
  );
}

export function useApp() {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error("useApp outside provider");
  return ctx;
}
