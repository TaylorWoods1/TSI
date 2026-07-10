import { useMemo } from "react";
import { Canvas } from "@react-three/fiber";
import { Grid, Line, OrbitControls } from "@react-three/drei";
import * as THREE from "three";
import type { Vec3 } from "../api/client";

THREE.Object3D.DEFAULT_UP.set(0, 0, 1);

export type SceneData = {
  anchors: Vec3[];
  dolly: Vec3;
  lengths: number[];
  workspace?: Vec3[];
};

type Props = {
  scene: SceneData;
  onAnchorDrag?: (index: number, pos: Vec3) => void;
  draggable?: boolean;
};

function AnchorHandle({
  index,
  position,
  onDrag,
  draggable,
}: {
  index: number;
  position: [number, number, number];
  onDrag?: (index: number, pos: Vec3) => void;
  draggable?: boolean;
}) {
  return (
    <mesh
      position={position}
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
      <sphereGeometry args={[0.18, 16, 16]} />
      <meshStandardMaterial color="#3dd6c6" emissive="#1a6b62" emissiveIntensity={0.4} />
    </mesh>
  );
}

function Cables({
  anchors,
  dolly,
}: {
  anchors: Vec3[];
  dolly: Vec3;
}) {
  const d = [dolly.x, dolly.y, dolly.z] as [number, number, number];
  return (
    <>
      {anchors.map((a, i) => (
        <Line
          key={i}
          points={[
            [a.x, a.y, a.z],
            d,
          ]}
          color="#8a9bab"
          lineWidth={1.5}
          transparent
          opacity={0.85}
        />
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
        />
      ))}
      <mesh position={dollyPos}>
        <octahedronGeometry args={[0.22, 0]} />
        <meshStandardMaterial color="#f4a261" emissive="#7a4a20" emissiveIntensity={0.35} />
      </mesh>
      <Cables anchors={scene.anchors} dolly={scene.dolly} />
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
