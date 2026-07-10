import { useMemo } from "react";
import { Canvas } from "@react-three/fiber";
import { Grid, Line, OrbitControls } from "@react-three/drei";
import * as THREE from "three";
import type { Vec3 } from "../api/client";

THREE.Object3D.DEFAULT_UP.set(0, 0, 1);

export type SceneData = {
  anchors: Vec3[];
  dolly: Vec3;
  attachments?: Vec3[];
  lengths: number[];
  cable_paths?: Vec3[][];
  unit_pulls?: Vec3[];
  model?: string;
  workspace?: Vec3[];
  show_pulls?: boolean;
};

type Props = {
  scene: SceneData;
  onAnchorDrag?: (index: number, pos: Vec3) => void;
  draggable?: boolean;
};

const CABLE_COLORS = ["#8a9bab", "#7eb8da", "#c9ada7", "#a8dadc", "#e9c46a", "#90be6d"];

function AnchorHandle({
  index,
  position,
  onDrag,
  draggable,
  isPulley,
}: {
  index: number;
  position: [number, number, number];
  onDrag?: (index: number, pos: Vec3) => void;
  draggable?: boolean;
  isPulley?: boolean;
}) {
  return (
    <group position={position}>
      {isPulley && (
        <mesh rotation={[Math.PI / 2, 0, 0]}>
          <torusGeometry args={[0.12, 0.025, 8, 24]} />
          <meshStandardMaterial color="#4a6fa5" emissive="#1a3050" emissiveIntensity={0.3} />
        </mesh>
      )}
      <mesh
        onPointerDown={(e) => {
          if (!draggable || !onDrag) return;
          e.stopPropagation();
          (e.target as THREE.Object3D).setPointerCapture(e.pointerId);
        }}
        onPointerUp={(e) => {
          if (!draggable) return;
          (e.target as THREE.Object3D).releasePointerCapture(e.pointerId);
        }}
        onPointerMove={(e) => {
          if (!draggable || !onDrag || !(e.target as THREE.Object3D).hasPointerCapture(e.pointerId))
            return;
          onDrag(index, { x: e.point.x, y: e.point.y, z: position[2] });
        }}
      >
        <sphereGeometry args={[isPulley ? 0.1 : 0.18, 16, 16]} />
        <meshStandardMaterial color="#3dd6c6" emissive="#1a6b62" emissiveIntensity={0.4} />
      </mesh>
    </group>
  );
}

function CablePolyline({ points, color }: { points: Vec3[]; color: string }) {
  const linePoints = useMemo(
    () => points.map((p) => [p.x, p.y, p.z] as [number, number, number]),
    [points],
  );
  if (linePoints.length < 2) return null;
  return <Line points={linePoints} color={color} lineWidth={2} transparent opacity={0.9} />;
}

function Cables({
  scene,
}: {
  scene: SceneData;
}) {
  if (scene.cable_paths && scene.cable_paths.length > 0) {
    return (
      <>
        {scene.cable_paths.map((path, i) => (
          <CablePolyline key={i} points={path} color={CABLE_COLORS[i % CABLE_COLORS.length]} />
        ))}
      </>
    );
  }
  const targets =
    scene.attachments && scene.attachments.length === scene.anchors.length
      ? scene.attachments
      : scene.anchors.map(() => scene.dolly);
  return (
    <>
      {scene.anchors.map((a, i) => (
        <Line
          key={i}
          points={[
            [a.x, a.y, a.z],
            [targets[i].x, targets[i].y, targets[i].z],
          ]}
          color={CABLE_COLORS[i % CABLE_COLORS.length]}
          lineWidth={1.5}
          transparent
          opacity={0.85}
        />
      ))}
    </>
  );
}

function PullArrows({ attachments, pulls }: { attachments: Vec3[]; pulls: Vec3[] }) {
  return (
    <>
      {attachments.map((att, i) => {
        const u = pulls[i];
        if (!u) return null;
        const len = Math.hypot(u.x, u.y, u.z);
        if (len < 1e-9) return null;
        const scale = 0.35 / len;
        const tip = { x: att.x + u.x * scale, y: att.y + u.y * scale, z: att.z + u.z * scale };
        return (
          <Line
            key={i}
            points={[
              [att.x, att.y, att.z],
              [tip.x, tip.y, tip.z],
            ]}
            color="#ff6b6b"
            lineWidth={2.5}
          />
        );
      })}
    </>
  );
}

function AttachmentMarkers({ attachments }: { attachments: Vec3[] }) {
  return (
    <>
      {attachments.map((p, i) => (
        <mesh key={i} position={[p.x, p.y, p.z]}>
          <sphereGeometry args={[0.06, 12, 12]} />
          <meshStandardMaterial color="#e76f51" emissive="#5a2010" emissiveIntensity={0.25} />
        </mesh>
      ))}
    </>
  );
}

function WorkspaceCloud({ points }: { points: Vec3[] }) {
  const positions = useMemo(() => {
    const arr = new Float32Array(points.length * 3);
    points.forEach((p, i) => {
      arr[i * 3] = p.x;
      arr[i * 3 + 1] = p.y;
      arr[i * 3 + 2] = p.z;
    });
    return arr;
  }, [points]);

  if (points.length === 0) return null;

  return (
    <points>
      <bufferGeometry>
        <bufferAttribute attach="attributes-position" args={[positions, 3]} />
      </bufferGeometry>
      <pointsMaterial color="#f4a261" size={0.08} transparent opacity={0.55} />
    </points>
  );
}

function SceneContent({ scene, onAnchorDrag, draggable }: Props) {
  const dollyPos = [scene.dolly.x, scene.dolly.y, scene.dolly.z] as [number, number, number];
  const isPulley = scene.model === "pulley";
  const attachments =
    scene.attachments && scene.attachments.length > 0 ? scene.attachments : [scene.dolly];

  return (
    <>
      <color attach="background" args={["#0a0f14"]} />
      <ambientLight intensity={0.45} />
      <directionalLight position={[8, 6, 12]} intensity={0.9} />
      <Grid
        args={[24, 24]}
        cellSize={1}
        cellColor="#1e2a33"
        sectionColor="#2a3d4a"
        fadeDistance={30}
        rotation={[Math.PI / 2, 0, 0]}
        position={[0, 0, 0]}
      />
      {scene.anchors.map((a, i) => (
        <AnchorHandle
          key={i}
          index={i}
          position={[a.x, a.y, a.z]}
          onDrag={onAnchorDrag}
          draggable={draggable}
          isPulley={isPulley}
        />
      ))}
      <mesh position={dollyPos}>
        <octahedronGeometry args={[0.22, 0]} />
        <meshStandardMaterial color="#f4a261" emissive="#7a4a20" emissiveIntensity={0.35} />
      </mesh>
      {scene.attachments && scene.attachments.length > 1 && (
        <AttachmentMarkers attachments={scene.attachments} />
      )}
      <Cables scene={scene} />
      {scene.show_pulls && scene.unit_pulls && (
        <PullArrows attachments={attachments} pulls={scene.unit_pulls} />
      )}
      {scene.workspace && <WorkspaceCloud points={scene.workspace} />}
      <OrbitControls makeDefault target={dollyPos} />
    </>
  );
}

export default function RobotScene(props: Props) {
  return (
    <Canvas camera={{ position: [12, -12, 10], fov: 50, up: [0, 0, 1] }}>
      <SceneContent {...props} />
    </Canvas>
  );
}
