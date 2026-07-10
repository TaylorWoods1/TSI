import { useCallback, useEffect, useRef, useState } from "react";
import * as api from "../api/client";
import type { SceneData } from "../scene/RobotScene";

const DEBOUNCE_MS = 80;

type Options = {
  orientationRv?: [number, number, number];
  workspace?: api.Vec3[];
  showPulls?: boolean;
  enabled?: boolean;
};

export function useSceneSnapshot(dolly: api.Vec3, options: Options = {}) {
  const { orientationRv, workspace, showPulls = false, enabled = true } = options;
  const [scene, setScene] = useState<SceneData | null>(null);
  const [sceneLoading, setSceneLoading] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const reqIdRef = useRef(0);

  const refreshScene = useCallback(
    async (pos: api.Vec3, ori?: [number, number, number]) => {
      const id = ++reqIdRef.current;
      setSceneLoading(true);
      try {
        const snap = await api.sceneSnapshot([pos.x, pos.y, pos.z], ori);
        if (id !== reqIdRef.current) return;
        const data = api.sceneSnapshotToSceneData(snap);
        setScene({
          ...data,
          workspace,
          show_pulls: showPulls,
        });
      } catch {
        if (id !== reqIdRef.current) return;
        setScene({
          anchors: [],
          dolly: pos,
          lengths: [],
          workspace,
          show_pulls: showPulls,
        });
      } finally {
        if (id === reqIdRef.current) setSceneLoading(false);
      }
    },
    [workspace, showPulls],
  );

  useEffect(() => {
    if (!enabled) return;
    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => {
      void refreshScene(dolly, orientationRv);
    }, DEBOUNCE_MS);
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, [dolly.x, dolly.y, dolly.z, orientationRv, enabled, refreshScene]);

  return { scene, sceneLoading, refreshScene };
}
