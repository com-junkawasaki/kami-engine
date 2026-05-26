import type { HumanoidBoneName, RotationAxis, JointLimitsMap, PosePreset } from '../types/bone.js';
import type { KamiWasmExports } from '../types/engine.js';
import { JOINT_LIMITS, clampBoneDeg } from '../data/joint-limits.js';

/** Options for creating a bone controller. */
export interface BoneControllerOpts {
  kami: KamiWasmExports | null;
  jointLimits?: JointLimitsMap;
  enforceConstraints?: boolean;
}

/** Bone rotation state per bone. */
export type BoneRotationMap = Map<string, { x: number; y: number; z: number }>;

/** All animated bone names for reset. */
const ANIMATED_BONES: string[] = [
  'head', 'neck', 'spine', 'chest', 'hips',
  'leftUpperArm', 'leftLowerArm', 'rightUpperArm', 'rightLowerArm',
  'leftUpperLeg', 'leftLowerLeg', 'rightUpperLeg', 'rightLowerLeg',
];

/**
 * Headless bone rotation controller with anatomical joint clamping.
 *
 * Manages per-bone Euler angle rotations and syncs to the KAMI Engine
 * VRM humanoid (Rust + wgpu via WASM, ADR-0031). Clamping is delegated
 * to Rust WASM when available, otherwise falls back to the TS
 * `clampBoneDeg` table. The three.js sync path was removed on
 * 2026-05-26 when the SDK went three-free.
 */
export function createBoneController(opts: BoneControllerOpts) {
  const limits = opts.jointLimits ?? JOINT_LIMITS;
  let enforce = $state(opts.enforceConstraints ?? true);
  let rotations: BoneRotationMap = $state(new Map());

  /** Set bone rotation on a single axis (degrees). Clamps if constraints enabled. */
  function setBone(name: string, axis: RotationAxis, degrees: number) {
    let deg = degrees;

    if (enforce) {
      if (opts.kami?.clampBone) {
        deg = opts.kami.clampBone(name, axis, degrees);
      } else {
        deg = clampBoneDeg(name, axis, degrees, limits);
      }
    }

    const current = rotations.get(name) ?? { x: 0, y: 0, z: 0 };
    current[axis] = deg;
    rotations.set(name, current);

    if (opts.kami?.setVrmBoneRotation) {
      const r = deg * Math.PI / 180;
      const half = r * 0.5;
      const s = Math.sin(half);
      const c = Math.cos(half);
      let qx = 0, qy = 0, qz = 0;
      if (axis === 'x') qx = s;
      else if (axis === 'y') qy = s;
      else qz = s;
      opts.kami.setVrmBoneRotation(name, qx, qy, qz, c);
    }
  }

  /** Apply a pose preset (reset first, then apply all bone rotations). */
  function applyPose(preset: PosePreset) {
    resetAll();
    for (const [bone, axes] of Object.entries(preset.bones)) {
      for (const [axis, deg] of Object.entries(axes)) {
        setBone(bone, axis as RotationAxis, deg as number);
      }
    }
  }

  /** Reset all bone rotations to 0. */
  function resetAll() {
    opts.kami?.resetVrmPose?.();
    rotations = new Map();
  }

  /** Update engine reference (after late init). */
  function updateEngines(kami: KamiWasmExports | null) {
    opts.kami = kami;
  }

  return {
    get rotations() { return rotations; },
    get enforceConstraints() { return enforce; },
    set enforceConstraints(v: boolean) { enforce = v; },
    setBone,
    applyPose,
    resetAll,
    updateEngines,
  };
}

export type BoneController = ReturnType<typeof createBoneController>;
