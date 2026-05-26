import type { KamiWasmExports } from '../types/engine.js';
import type { ExpressionPreset } from '../types/morph.js';

/** Options for creating a morph controller. */
export interface MorphControllerOpts {
  kami: KamiWasmExports | null;
  targetCount?: number;
}

/**
 * Headless morph target controller for VRM.
 *
 * Syncs morph weights to the KAMI Engine (Rust + wgpu via WASM). The
 * three.js sync path was removed on 2026-05-26 when the SDK went
 * three-free (see ADR-0031 + 40-engine/kami-engine/CLAUDE.md
 * "独自レンダラ禁止"). Uses Svelte 5 `$state` for reactivity.
 */
export function createMorphController(opts: MorphControllerOpts) {
  const count = opts.targetCount ?? 57;
  let weights = $state(new Float32Array(count));

  /** Set a single morph target weight (0–1). */
  function setMorph(index: number, weight: number) {
    if (index < 0 || index >= count) return;
    const w = Math.max(0, Math.min(1, weight));
    weights[index] = w;
    opts.kami?.setVrmMorph(index, w);
  }

  /** Apply a sparse weight map (index → weight). */
  function applyWeights(map: Record<number, number>) {
    for (const [idx, w] of Object.entries(map)) {
      setMorph(Number(idx), w);
    }
  }

  /** Apply an expression preset (reset + set preset morphs). */
  function applyPreset(preset: ExpressionPreset) {
    resetAll();
    applyWeights(preset.morphs);
  }

  /** Reset all morph weights to 0. */
  function resetAll() {
    opts.kami?.resetVrmMorphs();
    weights = new Float32Array(count);
  }

  /** Update engine reference (after late init). */
  function updateEngines(kami: KamiWasmExports | null) {
    opts.kami = kami;
  }

  return {
    get weights() { return weights; },
    setMorph,
    applyWeights,
    applyPreset,
    resetAll,
    updateEngines,
  };
}

export type MorphController = ReturnType<typeof createMorphController>;
