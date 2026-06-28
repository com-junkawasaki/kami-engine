(ns kami.mangaka.text
  "Work-agnostic, MULTILINGUAL manga text layer (ADR-2606282101).

  Lettering — dialogue / narration / SFX / thought — kept as DATA, separate from
  the (language-neutral) panel image, so one render serves every locale. The
  element shape follows ghosthacker's MangaText proto (kind/x/y/style) but the
  text is a LOCALE MAP ({:ja … :en …}); fukidashi types + JA vertical writing are
  first-class. Components return HICCUP, so reagent renders them in the browser
  and `kami.mangaka.hiccup` renders the SAME data to HTML for the static build.

  cljc + pure (no reagent/AWT/GPU here) — babashka-safe."
  (:require [clojure.string :as str]))

(def kinds   #{:dialogue :narration :sfx :thought})
(def bubbles #{:oval :jagged :cloud :square :wavy})
(def default-locale :ja)
(def fallback-order [:ja :en])

;; --- locale ----------------------------------------------------------------

(defn coerce-text
  "A bare string becomes {:ja s} (the project's existing JA-first convention);
  a locale map passes through."
  [t] (if (map? t) t {:ja t}))

(defn localize
  "Pick the best string for `locale` from a locale map, falling back through
  `fallback-order` then any present value. nil-safe."
  ([m locale] (localize m locale fallback-order))
  ([m locale order]
   (let [m (coerce-text m)]
     (or (get m locale) (some m order) (first (vals m))))))

(defn locales
  "All locales present across a seq of elements (for a language switcher)."
  [elements]
  (->> elements (mapcat (comp keys coerce-text :text)) distinct vec))

;; --- panel → ordered elements ----------------------------------------------

(defn panel->elements
  "Derive the ordered text-layer elements for a storyboard panel. Accepts the
  existing SIP/ghosthacker shape — :narration (string|locale-map), :dialogue
  [{:speaker :text}], and the new :sfx / :gh/sfx [{:text :bubble}] — and any
  explicit :text-layer (already in element form) which wins verbatim."
  [panel]
  (or (:text-layer panel)
      (let [narr (when-let [n (:narration panel)]
                   [{:kind :narration :text (coerce-text n)}])
            dlg  (for [d (:dialogue panel)]
                   {:kind :dialogue :speaker (:speaker d)
                    :text (coerce-text (:text d))
                    :bubble (or (:bubble d) :oval)})
            sfx  (for [s (or (:sfx panel) (:gh/sfx panel))]
                   {:kind :sfx :text (coerce-text (:text s))
                    :style (:style s)})]
        (vec (concat narr dlg sfx)))))

;; --- hiccup components (reagent-ready; pure data) ---------------------------

(defn- vertical? [locale] (= locale :ja))

(defn element->hiccup
  "One text element → hiccup. `side` (:l/:r) aims a dialogue bubble's tail."
  [locale {:keys [kind speaker text bubble] :as _el} side]
  (let [s (localize text locale)
        lang (name locale)
        vert (when (vertical? locale) "mk-vert")]
    (case kind
      :narration [:div.mk-cap {:lang lang} s]
      :sfx       [:div.mk-sfx {:lang lang :class vert} s]
      :thought   [:div.mk-bubble.mk-thought {:lang lang :class [vert (str "mk-" (name side))]} s]
      :dialogue  [:div.mk-bubble {:lang lang
                                  :class [vert (str "mk-" (name side))
                                          (str "mk-fuki-" (name (or bubble :oval)))]}
                  (when speaker [:span.mk-spk {:aria-hidden "true"} (name speaker)])
                  [:span.mk-line s]]
      [:div.mk-cap {:lang lang} s])))

(defn overlay
  "All of a panel's text elements → one positioned overlay hiccup, for laying
  over a rendered panel image. Dialogue/thought bubbles alternate sides for
  rhythm; narration pins top-left, SFX floats. Returns nil when empty."
  [locale elements]
  (when (seq elements)
    (let [bubble-idx (atom -1)]
      [:div.mk-ov
       (for [el elements
             :let [side (if (#{:dialogue :thought} (:kind el))
                          (if (even? (swap! bubble-idx inc)) :l :r)
                          :l)]]
         (element->hiccup locale el side))])))

;; --- default styles (mobile-first; the work may override) -------------------

(def css
  "Default overlay CSS: caption box, speech bubbles (with tail), SFX, and JA
  vertical writing (writing-mode: vertical-rl). Mobile-first."
  ".mk-fig{position:relative;display:block}
.mk-ov{position:absolute;inset:0;pointer-events:none;font-family:'Noto Sans JP','Hiragino Sans',sans-serif}
.mk-ov>*{pointer-events:auto}
.mk-cap{position:absolute;top:3%;left:3%;max-width:60%;background:rgba(255,253,247,.92);
  color:#222;border:1px solid #555;border-radius:3px;padding:.4em .6em;font-size:clamp(13px,3.4vw,17px);line-height:1.5}
.mk-bubble{position:absolute;bottom:6%;max-width:62%;background:#fff;color:#111;border:2.5px solid #111;
  border-radius:1.2em;padding:.5em .8em;font-size:clamp(14px,3.8vw,19px);line-height:1.55;
  box-shadow:0 2px 10px rgba(0,0,0,.35)}
.mk-bubble.mk-l{left:5%}.mk-bubble.mk-r{right:5%}
.mk-bubble::after{content:'';position:absolute;bottom:-12px;border:9px solid transparent;border-top-color:#111}
.mk-bubble.mk-l::after{left:18%}.mk-bubble.mk-r::after{right:18%}
.mk-bubble .mk-spk{display:block;font-size:.7em;color:#b07; font-weight:700;margin-bottom:.1em}
.mk-thought{border-style:dashed}
.mk-fuki-jagged{border-radius:.2em;clip-path:polygon(0 8%,6% 0,94% 0,100% 8%,100% 92%,94% 100%,6% 100%,0 92%)}
.mk-fuki-cloud{border-radius:50%/40%}
.mk-fuki-square{border-radius:.2em}
.mk-sfx{position:absolute;top:30%;right:6%;color:#fff;font-weight:900;
  font-size:clamp(26px,9vw,64px);line-height:1;letter-spacing:.02em;
  -webkit-text-stroke:2px #111;text-shadow:3px 3px 0 #111;transform:rotate(-8deg)}
.mk-vert{writing-mode:vertical-rl;text-orientation:mixed}
.mk-sfx.mk-vert{transform:rotate(0)}")
