<script lang="ts">
  import type { Snippet } from 'svelte';

  /**
   * IsekaiCanvas — Omniverse / PhysX / OpenUSD playable surface backed
   * by the `kami-app-isekai` WASM crate. Wires the kami-engine nv-compat
   * facade (`kami-usd` → omni.usd shape, `kami-genesis` → PxScene /
   * PxArticulationReducedCoordinate shape, ADR-2605261800 §D10.3).
   *
   * Constitutional invariants:
   *   - All NVIDIA-branded APIs are accessed through the `kami-*` facade
   *     namespace; NO direct PhysX / OmniKit / OpenUSD imports.
   *   - WebGPU primary, WebGL2 fallback (kami-render bootstrap policy).
   */
  interface Props {
    /**
     * Base URL of the `kami-app-isekai` WASM artifacts. Must contain
     * `kami_app_isekai.js` and `kami_app_isekai_bg.wasm` side by side.
     * Default: relative `/v2` matching the existing isekai deploy.
     */
    wasmBase?: string;
    /**
     * OpenUSD stage description. Pass an empty string (default) to use
     * the bundled `DEFAULT_ISEKAI_USDA` from the WASM artifact.
     */
    usda?: string;
    /** Optional CSS class on the wrapper div. */
    class?: string;
    /** Style overrides on the wrapper div. */
    style?: string;
    /** Background colour while the WASM/GPU is initialising. */
    bgColor?: string;
    /** Loading overlay snippet. */
    loading?: Snippet<[{ progress: number }]>;
    /** Error overlay snippet. */
    error?: Snippet<[{ message: string; retry: () => void }]>;
    /** Fired once the WASM run-loop is live. */
    onready?: (info: { banner: string; canvasId: string }) => void;
  }

  let {
    wasmBase = '/v2',
    usda = '',
    class: className,
    style,
    bgColor = '#0c0e14',
    loading: loadingSnippet,
    error: errorSnippet,
    onready,
  }: Props = $props();

  const uid = Math.random().toString(36).slice(2, 8);
  const canvasId = `kami-isekai-${uid}`;

  let initError = $state('');
  let booted = $state(false);
  let progress = $state(0);
  let banner = $state('');
  let retryCounter = $state(0);

  const retry = () => {
    initError = '';
    booted = false;
    retryCounter += 1;
  };

  $effect(() => {
    // Triggers on mount AND on retryCounter bump. Guard against SSR.
    if (typeof window === 'undefined') return;
    retryCounter; // dependency

    let cancelled = false;
    (async () => {
      try {
        progress = 0.1;
        const url = `${wasmBase.replace(/\/+$/, '')}/kami_app_isekai.js`;
        const mod = await import(/* @vite-ignore */ url);
        if (cancelled) return;
        progress = 0.5;
        await mod.default();
        if (cancelled) return;
        progress = 0.8;
        banner = typeof mod.isekaiOmniverseBanner === 'function'
          ? mod.isekaiOmniverseBanner()
          : '';
        const effectiveUsda =
          usda && usda.trim().length > 0
            ? usda
            : typeof mod.isekaiOmniverseDefaultUsda === 'function'
              ? mod.isekaiOmniverseDefaultUsda()
              : '';
        booted = true;
        onready?.({ banner, canvasId });
        // Launch the run-loop; this never resolves until the surface is
        // torn down by the host.
        await mod.runIsekaiOmniverse(canvasId, effectiveUsda);
      } catch (e: any) {
        if (cancelled) return;
        initError = String(e?.message ?? e);
        console.error('[IsekaiCanvas] init error:', e);
      }
    })();

    return () => {
      cancelled = true;
    };
  });
</script>

<div
  class={className ?? ''}
  style="position:relative;width:100%;height:100%;background:{bgColor};{style ?? ''}"
>
  <canvas
    id={canvasId}
    style="display:block;width:100%;height:100%;outline:none;"
    tabindex="0"
  ></canvas>

  {#if !booted && !initError}
    {#if loadingSnippet}
      {@render loadingSnippet({ progress })}
    {:else}
      <div
        style="position:absolute;inset:0;display:flex;align-items:center;justify-content:center;color:#9fc;font:12px ui-monospace,monospace;pointer-events:none;background:rgba(0,0,0,0.35)"
      >
        loading kami-engine nv-compat… ({Math.round(progress * 100)}%)
      </div>
    {/if}
  {/if}

  {#if initError}
    {#if errorSnippet}
      {@render errorSnippet({ message: initError, retry })}
    {:else}
      <div
        style="position:absolute;inset:0;display:flex;flex-direction:column;align-items:center;justify-content:center;color:#f88;font:12px ui-monospace,monospace;padding:16px;text-align:center;background:rgba(0,0,0,0.7)"
      >
        <div style="margin-bottom:8px">isekai init error</div>
        <div style="color:#fcc;max-width:480px">{initError}</div>
        <button
          onclick={retry}
          style="margin-top:12px;background:#1b3a5a;color:#cfe;border:1px solid #2a5378;border-radius:4px;padding:6px 12px;cursor:pointer;font:11px ui-monospace,monospace"
        >Retry</button>
      </div>
    {/if}
  {/if}

  {#if booted && banner}
    <div
      style="position:absolute;bottom:6px;left:8px;color:#a8e;font:10px ui-monospace,monospace;pointer-events:none;background:rgba(0,0,0,0.55);padding:2px 6px;border-radius:3px"
    >{banner}</div>
  {/if}
</div>
