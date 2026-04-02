import { useRef } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { useGameStore } from "../../store/gameStore";

const CAMERA_OFFSET = new THREE.Vector3(0, 18, 14);

/** Smooth follow camera that tracks the player */
export default function FollowCamera() {
  const cameraRef = useRef<THREE.PerspectiveCamera>(null!);

  useFrame(({ camera }) => {
    const { playerX, playerZ } = useGameStore.getState();
    const target = new THREE.Vector3(playerX, 0, playerZ).add(CAMERA_OFFSET);
    camera.position.lerp(target, 0.08);
    camera.lookAt(playerX, 0, playerZ);
  });

  return <perspectiveCamera ref={cameraRef} />;
}
