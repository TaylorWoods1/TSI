import { createContext, useContext, useState, type ReactNode } from "react";
import type { Vec3, Venue } from "./api/client";

export type TrajConfig = {
  start: [number, number, number];
  end: [number, number, number];
  segments: number;
};

type AppContextValue = {
  venue: Venue | null;
  setVenue: (v: Venue | null) => void;
  classify: string;
  setClassify: (c: string) => void;
  dolly: Vec3;
  setDolly: (d: Vec3) => void;
  traj: TrajConfig;
  setTraj: (t: TrajConfig) => void;
};

const AppContext = createContext<AppContextValue | null>(null);

export function AppProvider({ children }: { children: ReactNode }) {
  const [venue, setVenue] = useState<Venue | null>(null);
  const [classify, setClassify] = useState("—");
  const [dolly, setDolly] = useState<Vec3>({ x: 0, y: 0, z: 2 });
  const [traj, setTraj] = useState<TrajConfig>({
    start: [0, 0, 2],
    end: [0.5, 0, 2],
    segments: 8,
  });

  return (
    <AppContext.Provider
      value={{ venue, setVenue, classify, setClassify, dolly, setDolly, traj, setTraj }}
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
