(ns kami.binaural
  "L2 — binaural / spatial audio authored as EDN data → a backend-neutral
  spatialization IR. The clj layer is the *brain*: it turns a listener + a set of
  positioned sound sources into per-source binaural parameters (ITD / ILD / gains
  / delays). The *arm* is per-platform — `emit` lowers the IR to whatever the
  runtime can execute: a Web Audio node graph (browser), the `kami-audio` Rust
  mixer (native: iOS/Metal · Android · desktop), or a console mixer. Same EDN,
  any executor — the kami `audio.cljs` cue-bank pattern extended to 3D.

  This deliberately upgrades the old simplified dot-product pan (kami-audio's
  `spatialize`) to a physically-grounded *spherical-head* model, all in data:

    • ITD — Woodworth's spherical-head formula  itd = (a/c)(θ + sin θ),
      where θ is the lateral angle, a the head radius, c the speed of sound.
      Front/back symmetric and elevation-aware (θ shrinks as a source rises).
    • ILD — frequency-independent head-shadow approximation: the contralateral
      ear is attenuated proportionally to the lateral angle.
    • distance — OpenAL-style inverse / linear / exponential rolloff.

  EDN scene shape (everything optional except :sources):

    {:binaural/listener {:pos [0 0 0] :forward [0 0 -1] :up [0 1 0]}
     :binaural/hrtf     {:model :spherical-head :head-radius 0.0875 :max-ild-db 12.0}
     :binaural/rolloff  {:kind :inverse :ref 1.0 :max 100.0 :factor 1.0}
     :binaural/sources  [{:id :foot :cue :step :pos [3 0 -1] :gain 0.8}
                         {:id :bell :cue :ring :pos [0 1  4] :gain 1.0}]}

  `mix` is pure and serializable — the golden-test / record-replay surface."
  (:require [kami.math :as m]))

;; --- physical + default constants ------------------------------------------

(def ^:const speed-of-sound 343.0)          ; m/s, dry air ~20°C
(def ^:const default-head-radius 0.0875)    ; m, ~standard adult (KEMAR)

(def default-listener {:pos [0.0 0.0 0.0] :forward [0.0 0.0 -1.0] :up [0.0 1.0 0.0]})
(def default-hrtf     {:model :spherical-head :head-radius default-head-radius :max-ild-db 12.0})
(def default-rolloff  {:kind :inverse :ref 1.0 :max 100.0 :factor 1.0})

(def rolloff-kinds #{:inverse :linear :exponential :none})
(def hrtf-models   #{:spherical-head :pan-law})

;; --- validation -------------------------------------------------------------

(defn valid?
  "Throw with a precise reason if `scene` is malformed; return true otherwise.
  Mirrors kami.scene/valid? — fail fast at author time, not on the GPU/mixer."
  [scene]
  (let [{:keys [binaural/rolloff binaural/hrtf binaural/sources]} scene]
    (when (and rolloff (not (rolloff-kinds (:kind rolloff))))
      (throw (ex-info "binaural: unknown rolloff kind"
                      {:kind (:kind rolloff) :known rolloff-kinds})))
    (when (and hrtf (not (hrtf-models (:model hrtf))))
      (throw (ex-info "binaural: unknown hrtf model"
                      {:model (:model hrtf) :known hrtf-models})))
    (when-not (sequential? (or sources []))
      (throw (ex-info "binaural: :binaural/sources must be a sequence" {:sources sources})))
    (doseq [s sources]
      (when-not (and (vector? (:pos s)) (= 3 (count (:pos s))))
        (throw (ex-info "binaural: source needs a 3-vector :pos" {:source s}))))
    true))

;; --- listener basis ---------------------------------------------------------

(defn- listener-basis
  "Orthonormal {right up forward} for the listener (right-handed, robust to a
  non-orthogonal authored up vector)."
  [{:keys [forward up]}]
  (let [f  (m/normalize forward)
        r  (m/normalize (m/cross f up))
        u' (m/cross r f)]
    {:right r :up u' :forward f}))

;; --- distance rolloff -------------------------------------------------------

(defn distance-gain
  "Linear gain ∈ [0,1] for `dist` under a rolloff spec (OpenAL semantics)."
  [{:keys [kind ref max factor] :or {kind :inverse ref 1.0 max 100.0 factor 1.0}} dist]
  (let [d (m/clamp dist ref max)]
    (case kind
      :none        1.0
      :inverse     (/ ref (+ ref (* factor (- d ref))))
      :linear      (m/clamp (- 1.0 (* factor (/ (- d ref) (Math/max 1e-6 (- max ref))))) 0.0 1.0)
      :exponential (Math/pow (/ d ref) (- factor)))))

;; --- core spatialization (one source) --------------------------------------

(defn spatialize
  "Pure: listener + hrtf + rolloff + one source → binaural params.
  Returns {:distance :azimuth :elevation :lateral :itd-s :ild-db
           :gain-l :gain-r :delay-l-s :delay-r-s}. Angles in radians; the leading
  ear has delay 0 and the contralateral ear carries the ITD. gains fold in
  distance attenuation × source :gain."
  [listener hrtf rolloff source]
  (let [{:keys [right up forward]} (listener-basis listener)
        a        (:head-radius hrtf default-head-radius)
        max-ild  (:max-ild-db hrtf 12.0)
        rel      (m/v- (:pos source) (:pos listener))
        dist     (m/length rel)
        dir      (m/normalize rel)
        lateral  (m/clamp (m/dot dir right) -1.0 1.0)   ; +right, sin of lateral angle
        front    (m/dot dir forward)
        vert     (m/clamp (m/dot dir up) -1.0 1.0)
        azimuth  (Math/atan2 lateral front)
        elev     (Math/asin vert)
        theta    (Math/asin lateral)                    ; lateral angle, front/back symmetric
        itd      (* (/ a speed-of-sound) (+ theta (Math/sin theta)))  ; +→right leads
        ild-db   (* max-ild lateral)                    ; +→right louder
        dgain    (* (distance-gain rolloff dist) (:gain source 1.0))
        ;; attenuate only the contralateral (far) ear by |ILD|
        shadow   (Math/pow 10.0 (/ (- (Math/abs ild-db)) 20.0))
        right?   (>= lateral 0.0)
        gain-l   (* dgain (if right? shadow 1.0))
        gain-r   (* dgain (if right? 1.0 shadow))
        delay-l  (if (>= itd 0.0) itd 0.0)              ; right leads → delay left
        delay-r  (if (< itd 0.0) (- itd) 0.0)]
    {:distance dist :azimuth azimuth :elevation elev :lateral lateral
     :itd-s itd :ild-db ild-db
     :gain-l gain-l :gain-r gain-r :delay-l-s delay-l :delay-r-s delay-r}))

;; --- scene mix (the IR) -----------------------------------------------------

(defn mix
  "Pure: an EDN binaural scene → spatialization IR
     {:binaural/listener {...}
      :binaural/sources [{:source/id :source/cue :spatial {...}} ...]}
  Defaults are filled deterministically; source order is preserved."
  [scene]
  (valid? scene)
  (let [listener (merge default-listener (:binaural/listener scene))
        hrtf     (merge default-hrtf     (:binaural/hrtf scene))
        rolloff  (merge default-rolloff  (:binaural/rolloff scene))]
    {:binaural/listener listener
     :binaural/sources
     (vec (for [s (:binaural/sources scene)]
            {:source/id  (:id s)
             :source/cue (:cue s)
             :spatial    (spatialize listener hrtf rolloff s)}))}))

;; --- backend lowering (execution delegated per platform) --------------------

(defmulti emit
  "Lower the spatialization IR (from `mix`) to a backend-specific, executable
  descriptor. Dispatch on backend keyword. Execution itself happens in the
  runtime (Web Audio graph, kami-audio Rust mixer, console mixer) — this only
  produces the data that runtime consumes."
  (fn [backend _ir & _opts] backend))

(defmethod emit :web-audio
  ;; A node-graph recipe the cljs runtime builds: per source a DelayNode pair
  ;; (ITD) + GainNode pair (ILD + distance) + equal-power StereoPanner fallback.
  [_ ir & _]
  {:backend :web-audio
   :nodes (vec (for [{:source/keys [id cue] :keys [spatial]} (:binaural/sources ir)]
                 {:id id :cue cue
                  :delay-l (:delay-l-s spatial) :delay-r (:delay-r-s spatial)
                  :gain-l  (:gain-l spatial)    :gain-r  (:gain-r spatial)
                  :pan     (Math/sin (:azimuth spatial))}))})

(defmethod emit :native
  ;; Matches the kami-audio (Rust) AudioMixer::spatialize voice fields. ITD is
  ;; carried as an integer sample delay at the mixer's sample rate.
  [_ ir & [{:keys [sample-rate] :or {sample-rate 48000}}]]
  {:backend :native :sample-rate sample-rate
   :voices (vec (for [{:source/keys [id cue] :keys [spatial]} (:binaural/sources ir)]
                  {:id id :cue cue
                   :left-vol  (:gain-l spatial) :right-vol (:gain-r spatial)
                   :pan       (Math/sin (:azimuth spatial))
                   :delay-l-samples (long (Math/round (* (:delay-l-s spatial) sample-rate)))
                   :delay-r-samples (long (Math/round (* (:delay-r-s spatial) sample-rate)))}))})

(defmethod emit :default [backend ir & _]
  ;; Unknown backend: hand back the neutral IR so the caller can lower it itself.
  {:backend backend :ir ir})
