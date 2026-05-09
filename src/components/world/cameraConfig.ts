import * as THREE from "three";

/**
 * D2-style isometric camera offset.
 *
 * Pitch ~33° from horizontal (atan2(11, 17)) — close enough to the classic
 * 2:1 iso feel without losing readability of upright sprites.
 */
export const CAMERA_OFFSET = new THREE.Vector3(0, 11, 17);

/**
 * Tilt angle (radians) for fixed-iso prop sprites.
 *
 * A plane rotated around X by this angle has its normal pointing exactly at
 * the camera, so the sprite stays presented face-on regardless of player
 * position — true D2 look (no billboard wobble).
 */
export const SPRITE_TILT = Math.atan2(CAMERA_OFFSET.y, CAMERA_OFFSET.z);
