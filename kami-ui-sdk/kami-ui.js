/**
 * KAMI UI SDK v1.0.0
 * Nintendo-inspired UI components for kami-engine WebGPU applications.
 * Zero-dependency. All components are DOM-based overlays on top of WebGPU canvas.
 *
 * Usage:
 *   KamiUI.init({ font: 'Nunito' });
 *   const hud = KamiUI.StatusBar({ text: 'Loading...', position: 'top-left' });
 *   hud.setText('Ready');
 *
 * @license MIT
 * @see https://kami.etzhayyim.com/ui-sdk
 */
(function(root) {
"use strict";

// ━━━ Theme ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

const THEME = {
  bg: '#f0ead6',
  cardBg: 'rgba(255,255,255,0.92)',
  cardBorder: '#dfe6e9',
  textPrimary: '#2d3436',
  textSecondary: '#636e72',
  textMuted: '#b2bec3',
  shadow: '0 4px 16px rgba(0,0,0,0.08)',
  shadowSmall: '0 2px 8px rgba(0,0,0,0.06)',
  radius: '16px',
  radiusSmall: '12px',
  font: "'Nunito', system-ui, -apple-system, sans-serif",
  fontUrl: 'https://fonts.googleapis.com/css2?family=Nunito:wght@400;700;900&display=swap',
  // Nintendo Splatoon-inspired accent colors
  accent: {
    red: '#fa5757',
    blue: '#33bfff',
    green: '#66e673',
    gold: '#ffbf33',
    purple: '#b366f2',
    mint: '#26d9b3',
    orange: '#ff8c4d',
    pink: '#f272b3',
  },
};

let _fontLoaded = false;

function _loadFont() {
  if (_fontLoaded) return;
  _fontLoaded = true;
  const link = document.createElement('link');
  link.rel = 'stylesheet';
  link.href = THEME.fontUrl;
  document.head.appendChild(link);
}

function _applyBase(el, overrides) {
  el.style.fontFamily = THEME.font;
  el.style.boxSizing = 'border-box';
  if (overrides) Object.assign(el.style, overrides);
  return el;
}

// ━━━ StatusBar ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/**
 * Status bar — floating pill with title text.
 * @param {object} opts - { text, position: 'top-left'|'top-right'|'bottom-left'|'bottom-right' }
 */
function StatusBar(opts) {
  opts = opts || {};
  const el = document.createElement('div');
  _applyBase(el, {
    position: 'fixed', zIndex: '10',
    color: THEME.textPrimary, fontSize: '15px', fontWeight: '700',
    background: THEME.cardBg, padding: '10px 18px',
    borderRadius: THEME.radius, boxShadow: THEME.shadow,
    border: '2px solid ' + THEME.cardBorder,
    pointerEvents: 'none',
  });
  _positionElement(el, opts.position || 'top-left');
  el.textContent = opts.text || '';
  document.body.appendChild(el);

  // Animate entrance if KamiMotion available
  if (root.KamiMotion) root.KamiMotion.fadeIn(el);

  return {
    el,
    setText(t) { el.textContent = t; },
    remove() {
      if (root.KamiMotion) root.KamiMotion.fadeOut(el, { onComplete: () => el.remove() });
      else el.remove();
    },
  };
}

// ━━━ ControlHint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/**
 * Control hints — small floating bar showing keyboard/mouse shortcuts.
 * @param {object} opts - { hints: [{key, action}], position }
 */
function ControlHint(opts) {
  opts = opts || {};
  const el = document.createElement('div');
  _applyBase(el, {
    position: 'fixed', zIndex: '10',
    color: THEME.textSecondary, fontSize: '12px', fontWeight: '600',
    background: 'rgba(255,255,255,0.88)', padding: '8px 14px',
    borderRadius: THEME.radiusSmall, boxShadow: THEME.shadowSmall,
    display: 'flex', gap: '12px', alignItems: 'center',
    pointerEvents: 'none',
  });
  _positionElement(el, opts.position || 'bottom-left');

  const hints = opts.hints || [
    { key: 'Drag', action: 'pan' },
    { key: 'Scroll', action: 'zoom' },
    { key: 'Dbl-click', action: 'reset' },
  ];
  for (const h of hints) {
    const span = document.createElement('span');
    span.innerHTML = '<b style="color:' + THEME.textPrimary + '">' + h.key + '</b> ' + h.action;
    el.appendChild(span);
  }
  document.body.appendChild(el);

  return { el, remove() { el.remove(); } };
}

// ━━━ LabelOverlay ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/**
 * Label overlay — positioned text labels that track camera state.
 * Reads camera from window.__kami_cam_* globals (set by kami-web WASM).
 * @param {object} opts - { nodes: [{n, g, x, z}], maxLabels, canvasWidth, canvasHeight }
 */
function LabelOverlay(opts) {
  opts = opts || {};
  const container = document.createElement('div');
  container.style.cssText = 'position:absolute;top:0;left:0;width:100%;height:100%;pointer-events:none;overflow:hidden';
  document.body.appendChild(container);

  const MAX = opts.maxLabels || 300;
  const pool = [];
  for (let i = 0; i < MAX; i++) {
    const el = document.createElement('div');
    _applyBase(el, {
      position: 'absolute', whiteSpace: 'nowrap',
      transform: 'translate(-50%, -100%)',
      color: THEME.textPrimary, fontWeight: '700',
      textShadow: '0 1px 2px rgba(255,255,255,0.9), 0 0 4px rgba(255,255,255,0.7)',
      letterSpacing: '-0.02em',
      display: 'none',
    });
    container.appendChild(el);
    pool.push(el);
  }

  let nodes = opts.nodes || [];
  const W = opts.canvasWidth || window.innerWidth;
  const H = opts.canvasHeight || window.innerHeight;
  let running = true;

  function update() {
    if (!running) return;
    const camX = window.__kami_cam_x || 0;
    const camZ = window.__kami_cam_z || 0;
    const zoom = window.__kami_cam_zoom || 1000;

    const aspect = W / H;
    const halfW = zoom * aspect;
    const halfH = zoom;
    const vL = camX - halfW, vR = camX + halfW;
    const vT = camZ - halfH, vB = camZ + halfH;

    const fontSize = Math.max(8, Math.min(16, 18000 / zoom));
    let idx = 0;

    for (const node of nodes) {
      if (idx >= MAX) break;
      if (node.x < vL || node.x > vR || node.z < vT || node.z > vB) continue;

      const sx = ((node.x - vL) / (vR - vL)) * W;
      const sy = ((node.z - vT) / (vB - vT)) * H;
      const el = pool[idx];
      el.style.display = 'block';
      el.style.left = sx + 'px';
      el.style.top = (sy - 4) + 'px';
      el.textContent = node.n;
      el.style.fontSize = fontSize + 'px';
      idx++;
    }
    for (let i = idx; i < MAX; i++) pool[i].style.display = 'none';
    requestAnimationFrame(update);
  }
  requestAnimationFrame(update);

  return {
    container,
    setNodes(n) { nodes = n; },
    destroy() { running = false; container.remove(); },
  };
}

// ━━━ FileLoader ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/**
 * File loader button — styled file input.
 * @param {object} opts - { accept, position, parent, onLoad: (text, filename) => void }
 */
function FileLoader(opts) {
  opts = opts || {};
  const wrapper = document.createElement('div');
  if (opts.parent) {
    _applyBase(wrapper, {
      display: 'block',
      width: '100%',
    });
    opts.parent.appendChild(wrapper);
  } else {
    wrapper.style.cssText = 'position:fixed;z-index:10';
    _positionElement(wrapper, opts.position || 'top-right');
    document.body.appendChild(wrapper);
  }

  const input = document.createElement('input');
  input.type = 'file';
  input.accept = opts.accept || '.json';
  _applyBase(input, {
    fontSize: '13px', padding: '8px 12px',
    borderRadius: THEME.radiusSmall,
    border: '2px solid ' + THEME.cardBorder,
    background: '#fff', cursor: 'pointer',
    width: '100%',
    color: THEME.textPrimary,
  });
  input.addEventListener('change', e => {
    const file = e.target.files[0];
    if (!file || !opts.onLoad) return;
    const reader = new FileReader();
    reader.onload = ev => opts.onLoad(ev.target.result, file.name);
    reader.readAsText(file);
  });
  wrapper.appendChild(input);

  return { el: wrapper, remove() { wrapper.remove(); } };
}

// ━━━ Toast ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/**
 * Toast notification — temporary popup message.
 * @param {string} msg - message text
 * @param {object} opts - { duration: ms, type: 'info'|'success'|'error' }
 */
function Toast(msg, opts) {
  opts = opts || {};
  const colors = { info: THEME.accent.blue, success: THEME.accent.green, error: THEME.accent.red };
  const color = colors[opts.type || 'info'] || THEME.accent.blue;

  const el = document.createElement('div');
  _applyBase(el, {
    position: 'fixed', bottom: '60px', left: '50%', transform: 'translateX(-50%)',
    zIndex: '20',
    color: '#fff', fontSize: '14px', fontWeight: '700',
    background: color, padding: '10px 24px',
    borderRadius: '24px', boxShadow: '0 4px 20px rgba(0,0,0,0.15)',
    transition: 'opacity 0.3s, transform 0.3s',
    opacity: '0',
  });
  el.textContent = msg;
  document.body.appendChild(el);

  if (root.KamiMotion) {
    root.KamiMotion.popIn(el);
    setTimeout(() => root.KamiMotion.popOut(el, { onComplete: () => el.remove() }), opts.duration || 2000);
  } else {
    requestAnimationFrame(() => { el.style.opacity = '1'; });
    setTimeout(() => { el.style.opacity = '0'; setTimeout(() => el.remove(), 300); }, opts.duration || 2000);
  }
  // Sound
  if (root.KamiSound) {
    const soundMap = { info: 'pop', success: 'success', error: 'error' };
    root.KamiSound.play(soundMap[opts.type || 'info'] || 'pop');
  }

  return { el };
}

// ━━━ Badge ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/**
 * Badge — small colored pill (for counts, tags).
 * @param {string} text
 * @param {object} opts - { color, parent }
 */
function Badge(text, opts) {
  opts = opts || {};
  const el = document.createElement('span');
  _applyBase(el, {
    display: 'inline-block',
    color: '#fff', fontSize: '11px', fontWeight: '900',
    background: opts.color || THEME.accent.blue,
    padding: '2px 8px', borderRadius: '10px',
    lineHeight: '1.4',
  });
  el.textContent = text;
  if (opts.parent) opts.parent.appendChild(el);
  return { el, setText(t) { el.textContent = t; } };
}

// ━━━ Legend ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/**
 * Legend — color-keyed list (for edge types, groups).
 * @param {object} opts - { items: [{color, label}], position }
 */
function Legend(opts) {
  opts = opts || {};
  const el = document.createElement('div');
  _applyBase(el, {
    position: 'fixed', zIndex: '10',
    background: THEME.cardBg, padding: '10px 14px',
    borderRadius: THEME.radiusSmall, boxShadow: THEME.shadowSmall,
    border: '1px solid ' + THEME.cardBorder,
    fontSize: '11px', fontWeight: '600', color: THEME.textSecondary,
    display: 'flex', flexDirection: 'column', gap: '4px',
  });
  _positionElement(el, opts.position || 'bottom-right');

  for (const item of (opts.items || [])) {
    const row = document.createElement('div');
    row.style.display = 'flex';
    row.style.alignItems = 'center';
    row.style.gap = '6px';
    const dot = document.createElement('span');
    dot.style.cssText = 'width:10px;height:10px;border-radius:3px;flex-shrink:0;background:' + item.color;
    const label = document.createElement('span');
    label.textContent = item.label;
    row.appendChild(dot);
    row.appendChild(label);
    el.appendChild(row);
  }
  document.body.appendChild(el);

  // Stagger-animate legend items
  if (root.KamiMotion) {
    const rows = el.querySelectorAll('div');
    root.KamiMotion.stagger(rows, {}, { delay: 40, animation: 'fadeIn' });
  }

  return { el, remove() { el.remove(); } };
}

// ━━━ Panel ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/**
 * Panel — card container with optional header and body area.
 * @param {object} opts - { title, subtitle, parent, bodyStyles }
 */
function Panel(opts) {
  opts = opts || {};
  const el = document.createElement('section');
  _applyBase(el, {
    background: THEME.cardBg,
    border: '1px solid ' + THEME.cardBorder,
    borderRadius: THEME.radius,
    boxShadow: THEME.shadowSmall,
    padding: '14px',
    display: 'grid',
    gap: '12px',
  });

  let titleEl = null;
  let subtitleEl = null;
  if (opts.title || opts.subtitle) {
    const header = document.createElement('div');
    _applyBase(header, { display: 'grid', gap: '4px' });
    if (opts.title) {
      titleEl = document.createElement('div');
      _applyBase(titleEl, {
        color: THEME.textSecondary,
        fontSize: '11px',
        fontWeight: '900',
        letterSpacing: '0.08em',
        textTransform: 'uppercase',
      });
      titleEl.textContent = opts.title;
      header.appendChild(titleEl);
    }
    if (opts.subtitle) {
      subtitleEl = document.createElement('div');
      _applyBase(subtitleEl, {
        color: THEME.textMuted,
        fontSize: '12px',
        fontWeight: '700',
      });
      subtitleEl.textContent = opts.subtitle;
      header.appendChild(subtitleEl);
    }
    el.appendChild(header);
  }

  const body = document.createElement('div');
  _applyBase(body, Object.assign({ display: 'grid', gap: '10px' }, opts.bodyStyles || {}));
  el.appendChild(body);

  if (opts.parent) opts.parent.appendChild(el);
  return { el, body, titleEl, subtitleEl };
}

// ━━━ Field ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/**
 * Field — labeled form field wrapper.
 * @param {object} opts - { label, hint, control, parent }
 */
function Field(opts) {
  opts = opts || {};
  const el = document.createElement('label');
  _applyBase(el, {
    display: 'grid',
    gap: '6px',
    color: THEME.textPrimary,
  });

  if (opts.label) {
    const labelEl = document.createElement('span');
    _applyBase(labelEl, {
      color: THEME.textSecondary,
      fontSize: '12px',
      fontWeight: '800',
    });
    labelEl.textContent = opts.label;
    el.appendChild(labelEl);
  }

  if (opts.control) el.appendChild(opts.control);

  if (opts.hint) {
    const hintEl = document.createElement('span');
    _applyBase(hintEl, {
      color: THEME.textMuted,
      fontSize: '11px',
      fontWeight: '700',
    });
    hintEl.textContent = opts.hint;
    el.appendChild(hintEl);
  }

  if (opts.parent) opts.parent.appendChild(el);
  return { el };
}

// ━━━ Button ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/**
 * Button — SDK-styled button with primary/ghost variants.
 * @param {object} opts - { text, variant, color, active, parent, onClick }
 */
function Button(opts) {
  opts = opts || {};
  const el = document.createElement('button');
  el.type = 'button';
  const baseColor = opts.color || THEME.accent.blue;
  const active = !!opts.active;
  _applyBase(el, {
    appearance: 'none',
    border: '1px solid ' + (active ? baseColor : THEME.cardBorder),
    background: active ? baseColor : (opts.variant === 'primary' ? THEME.textPrimary : '#fff'),
    color: active || opts.variant === 'primary' ? '#fff' : THEME.textPrimary,
    borderRadius: THEME.radiusSmall,
    padding: '10px 12px',
    fontSize: '12px',
    fontWeight: '800',
    cursor: 'pointer',
    transition: 'transform 120ms ease, box-shadow 120ms ease, background 120ms ease',
    boxShadow: active ? THEME.shadowSmall : 'none',
  });
  el.textContent = opts.text || 'Button';
  el.addEventListener('mouseenter', () => {
    el.style.transform = 'translateY(-1px)';
    el.style.boxShadow = THEME.shadowSmall;
  });
  el.addEventListener('mouseleave', () => {
    el.style.transform = 'translateY(0)';
    el.style.boxShadow = active ? THEME.shadowSmall : 'none';
  });
  if (opts.onClick) el.addEventListener('click', opts.onClick);
  if (opts.parent) opts.parent.appendChild(el);

  return {
    el,
    setActive(nextActive) {
      el.style.borderColor = nextActive ? baseColor : THEME.cardBorder;
      el.style.background = nextActive ? baseColor : (opts.variant === 'primary' ? THEME.textPrimary : '#fff');
      el.style.color = nextActive || opts.variant === 'primary' ? '#fff' : THEME.textPrimary;
      el.style.boxShadow = nextActive ? THEME.shadowSmall : 'none';
    },
  };
}

// ━━━ Helpers ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

function _positionElement(el, pos) {
  const m = '16px';
  switch (pos) {
    case 'top-left':     el.style.top = m; el.style.left = m; break;
    case 'top-right':    el.style.top = m; el.style.right = m; break;
    case 'bottom-left':  el.style.bottom = m; el.style.left = m; break;
    case 'bottom-right': el.style.bottom = m; el.style.right = m; break;
    case 'top-center':   el.style.top = m; el.style.left = '50%'; el.style.transform = 'translateX(-50%)'; break;
    case 'bottom-center': el.style.bottom = m; el.style.left = '50%'; el.style.transform = 'translateX(-50%)'; break;
  }
}

// ━━━ Init ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/**
 * Initialize KAMI UI — loads font, applies base styles.
 * @param {object} opts - { font, bg }
 */
function init(opts) {
  opts = opts || {};
  if (opts.font) THEME.font = "'" + opts.font + "', system-ui, sans-serif";
  if (opts.bg) THEME.bg = opts.bg;
  _loadFont();
  document.body.style.fontFamily = THEME.font;
  document.body.style.background = THEME.bg;
}

// ━━━ Public API ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

root.KamiUI = {
  VERSION: '1.0.0',
  THEME,
  init,
  StatusBar,
  ControlHint,
  LabelOverlay,
  FileLoader,
  Toast,
  Badge,
  Legend,
  Panel,
  Field,
  Button,
};

})(typeof globalThis !== 'undefined' ? globalThis : typeof window !== 'undefined' ? window : this);
