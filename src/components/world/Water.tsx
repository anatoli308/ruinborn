/** Water patches on the map */
export default function Water() {
  return (
    <>
      <mesh rotation={[-Math.PI / 2, 0, 0]} position={[-28, 0.04, 18]}>
        <circleGeometry args={[12, 32]} />
        <meshStandardMaterial color="#1a5276" roughness={0.2} metalness={0.3} transparent opacity={0.75} />
      </mesh>
      <mesh rotation={[-Math.PI / 2, 0, 0]} position={[22, 0.04, -18]}>
        <circleGeometry args={[8, 32]} />
        <meshStandardMaterial color="#1a5276" roughness={0.2} metalness={0.3} transparent opacity={0.75} />
      </mesh>
    </>
  );
}
