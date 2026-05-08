import { useRef } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { useGameStore } from "../../store/gameStore";

const CAMERA_OFFSET = new THREE.Vector3(0, 18, 14);

/** Smooth follow camera that tracks the player */
export default function FollowCamera() {
  const cameraRef = useRef<THREE.PerspectiveCamera>(null!);
  const targetBase = useRef(new THREE.Vector3());
  const target = useRef(new THREE.Vector3());

  useFrame(({ camera, scene }) => {
    const playerObject = scene.getObjectByName("local-player");
    const playerPos = playerObject?.position;
    if (playerPos) {
      targetBase.current.copy(playerPos);
    } else {
      const { playerX, playerZ } = useGameStore.getState();
      targetBase.current.set(playerX, 0, playerZ);
    }

    target.current.copy(targetBase.current).add(CAMERA_OFFSET);

    camera.position.lerp(target.current, 0.08);
    camera.lookAt(targetBase.current.x, 0, targetBase.current.z);
  });

  return <perspectiveCamera ref={cameraRef} />;
}
