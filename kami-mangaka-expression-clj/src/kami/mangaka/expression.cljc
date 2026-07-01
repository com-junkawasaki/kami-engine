(ns kami.mangaka.expression
  "Work-agnostic manga EXPRESSION patterns (ADR-2607012100, Tier-1 mangaka).

  漫画的表現 — 「キャラクターごとに、表情・姿勢・吹き出しの形・文字の薄さ・大きさ・
  背景トーンを変える」— を DATA から決める純粋レイヤ。HUNTER×HUNTER 王位継承戦編の
  コマ観察を出典にしたパターン (`resources/mangaka_expression_patterns.edn`) を、

    archetype(キャラ類型) ← register(セリフ種別) ← expression-cue(感情) ← intensity(強度)

  の順に merge して 1 行分のスタイル (`resolve-style`) を出す。`analyze-line` /
  `analyze-panel` / `analyze-page` はそれを storyboard に当てる (langgraph の
  ai.gftd.mangaka.analyzeExpression がこれを呼ぶ)。

  ここは SEMANTIC タグだけを吐く純 cljc — 見た目への写像 (opacity/font-size/screentone)
  は renderer 側 (kami.mangaka.text の hiccup+CSS / kami.mangaka.page の Java2D) が持つ。
  babashka-safe / JVM・cljs・WASM 可搬 (host interop は reader-conditional のみ)。"
  (:require [clojure.string :as str]
            #?(:clj [clojure.edn :as edn])
            #?(:clj [clojure.java.io :as io])))

;; ---------------------------------------------------------------------------
;; Vocabulary — the edn :vocab の canonical 版 (test で edn と lock-step)
;; ---------------------------------------------------------------------------

(def registers
  "セリフ/文字レジスタ。text.cljc の :kind を包含し、群衆ざわめき/名札を足す。"
  #{:speech :shout :whisper :thought :monologue :narration :chatter :nameplate :sfx})

(def bubbles
  "吹き出しの形。kami.mangaka.text/bubbles ∪ {:spike(叫び) :burst(放射)}。"
  #{:oval :jagged :cloud :square :wavy :spike :burst})

(def expressions
  "表情。kami.mangaka.scene の Rust Expression variant と一致。"
  #{"Neutral" "Happy" "Angry" "Sad" "Surprised" "Determined" "Pained" "Smirk"})

(def font-roles
  "写植ロール。ai-gftd-mangaka の manga-font-palette (fontRole) に対応。"
  #{:antigochi :gothic :mincho :bold-mincho :maru :kyokasho :brush :handwritten :digital :horror})

(def weights
  "文字の薄さ↔太さ。薄→濃 の順序ベクタ (index が段数)。"
  [:faint :light :regular :bold :heavy])

(def tones
  "背景トーン (screentone / 集中線 / フラッシュ / 群衆シルエット など)。"
  #{:none :flat-white :dot :gradient :focus-lines :radial-burst :flash
    :vignette-dark :hatching :crowd-silhouette})

(def postures
  #{:upright :lean-in :arm-raised :recoil :slump :point :cross-arms :turn-away})

(def marks
  #{:sweat :anger-vein :shock :tear :sparkle :question :exclaim})

(def dramatic-tones
  "強度が要る演出トーン (低 intensity では減衰させる対象)。"
  #{:flash :focus-lines :radial-burst :vignette-dark})

;; ---------------------------------------------------------------------------
;; Pattern library load (clj — resource; cljs は analyze に patterns を渡す)
;; ---------------------------------------------------------------------------

(def default-patterns-resource "mangaka_expression_patterns.edn")

#?(:clj
   (defn load-patterns
     "Load the bundled pattern library from the classpath (or an override name)."
     ([] (load-patterns default-patterns-resource))
     ([resource]
      (with-open [r (io/reader (io/resource resource))]
        (edn/read (java.io.PushbackReader. r))))))

;; ---------------------------------------------------------------------------
;; Emotion → Expression (kami.mangaka.scene/expression-of と同表; scene は JVM の
;; ため cljc からは require せず複製。両者が乖離しないよう test で照合する。)
;; ---------------------------------------------------------------------------

(defn expression-of
  "感情語を Expression variant に正規化。未知は \"Neutral\"。"
  [name]
  (if (expressions name)
    name
    (case (str/lower-case (str name))
      ("happy" "joy" "smile" "喜" "笑")        "Happy"
      ("angry" "rage" "怒" "激昂")             "Angry"
      ("sad" "sorrow" "grief" "哀" "悲")       "Sad"
      ("surprised" "surprise" "shock" "驚")    "Surprised"
      ("determined" "resolve" "focus" "決意")  "Determined"
      ("pained" "pain" "hurt" "苦")            "Pained"
      ("smirk" "smug" "冷笑" "不敵")           "Smirk"
      "Neutral")))

;; ---------------------------------------------------------------------------
;; helpers
;; ---------------------------------------------------------------------------

(defn- clamp [lo hi x] (max lo (min hi x)))

(defn weight-index [w] (or (first (keep-indexed #(when (= %2 w) %1) weights)) 2))

(defn weight-bump
  "Advance a weight `n` steps toward :heavy (clamped)."
  [w n]
  (get weights (clamp 0 (dec (count weights)) (+ (weight-index w) n)) w))

(defn- prune [m] (into {} (remove (comp nil? val) m)))

(defn infer-register
  "Guess a register from an authored line when none is given. JA/EN heuristics:
  kind wins; then bracketed → whisper, 2+ bang/interrobang → shout, else speech."
  [{:keys [kind text]}]
  (let [s (str (if (map? text) (or (:ja text) (first (vals text))) text))]
    (cond
      (contains? #{:narration :monologue :thought :sfx :chatter :nameplate :whisper :shout} kind) kind
      (re-find #"（.*）|\(.*\)|〈.*〉" s)                    :whisper
      (or (re-find #"[!！][!！]|[?？][!！]|[!！][?？]" s)
          (re-find #"[!！]\s*$" s))                          :shout
      :else                                                 :speech)))

;; ---------------------------------------------------------------------------
;; resolve-style — archetype ← register ← expression-cue ← intensity
;; ---------------------------------------------------------------------------

(defn resolve-style
  "一行分の漫画表現スタイルを解決する。返り値:
   {:expression :register :bubble :font-role :weight :scale :tone :posture
    :marks :fx? :dashed? :invert? :rotate?}.
   `opts` = {:archetype kw :register kw :expression str :intensity double}."
  [patterns {:keys [archetype register expression intensity] :or {intensity 0.0}}]
  (let [arch-k (or archetype (:default-archetype patterns) :stoic)
        reg-k  (or register (:default-register patterns) :speech)
        arch   (or (get-in patterns [:archetypes arch-k])
                   (get-in patterns [:archetypes (:default-archetype patterns)])
                   {})
        reg    (or (get-in patterns [:registers reg-k])
                   (get-in patterns [:registers (:default-register patterns)])
                   {})
        expr   (expression-of (or expression (:expression arch) "Neutral"))
        cue    (get-in patterns [:expression-cues expr] {})
        ;; register が archetype に勝つ (吹き出し/太さ/大きさ/書体/トーンの主因)
        base   (merge arch reg)
        ;; 感情 → 姿勢/マーク/トーン傾向 の補正
        base   (cond-> base
                 (:posture cue)     (assoc :posture (:posture cue))
                 (seq (:marks cue)) (assoc :marks (vec (distinct (concat (:marks base) (:marks cue)))))
                 ;; トーンは register 指定 > archetype 指定 > 感情 tone-bias
                 (and (not (:tone reg)) (:tone-bias cue) (not= :none (:tone-bias cue)))
                 (assoc :tone (:tone-bias cue)))
        base   (assoc base :expression expr :register reg-k)
        ;; intensity 増幅
        {:keys [scale-gain scale-max weight-steps tone-threshold subtle-tone]
         :or   {scale-gain 0.0 scale-max 3.0 weight-steps [] tone-threshold 0.0 subtle-tone :dot}}
        (:intensity patterns)
        i      (clamp 0.0 1.0 (double intensity))
        scale  (clamp 0.1 scale-max (* (double (or (:scale base) 1.0)) (+ 1.0 (* scale-gain i))))
        bump   (reduce (fn [acc [thr n]] (if (>= i (double thr)) (max acc (long n)) acc)) 0 weight-steps)
        wt     (weight-bump (or (:weight base) :regular) bump)
        tone   (if (and (< i (double tone-threshold)) (dramatic-tones (:tone base)))
                 subtle-tone (:tone base))]
    (prune (assoc base :scale scale :weight wt :tone tone))))

;; ---------------------------------------------------------------------------
;; analyze — apply resolve-style to authored storyboard shapes
;; ---------------------------------------------------------------------------

(defn resolve-archetype
  "speaker (string|keyword) → archetype keyword via the cast map, else default."
  [char->archetype speaker patterns]
  (or (get char->archetype speaker)
      (get char->archetype (some-> speaker name))
      (get char->archetype (when speaker (keyword (name speaker))))
      (:default-archetype patterns)
      :stoic))

(defn analyze-line
  "Enrich one authored line (`{:speaker :text :bubble? :register? :emotion?
  :expression? :intensity? :kind?}`) with expression style. Explicit keys win;
  missing ones are filled from the resolved style. `char->archetype` maps a
  speaker to an archetype keyword."
  [patterns char->archetype {:keys [speaker register emotion expression intensity] :as line}]
  (let [reg   (or register (infer-register line))
        arch  (resolve-archetype char->archetype speaker patterns)
        style (resolve-style patterns {:archetype arch :register reg
                                       :expression (or expression emotion)
                                       :intensity (or intensity 0.0)})]
    (prune
     (merge
      ;; style-derived semantic tags
      (select-keys style [:posture :font-role :tone :marks :fx :dashed :invert :rotate])
      {:archetype arch
       :register reg
       :expression (:expression style)}
      ;; the authored line (explicit values win over style defaults) — but a key
      ;; carried as an *explicit nil* (e.g. an adapter that emits :register nil for
      ;; "unset") must NOT clobber the resolved default, so drop nils first.
      (prune line)
      ;; …and fill bubble/weight/scale from style when the line didn't set them
      {:bubble (or (:bubble line) (:bubble style))
       :weight (or (:weight line) (:weight style))
       :scale  (or (:scale line)  (:scale style))}))))

(def tone-salience
  "背景としての支配度。パネルは最も salient な行トーンを採る (叫び>群衆>網掛/グラデ>平)。"
  {:flash 6 :radial-burst 6 :focus-lines 5 :vignette-dark 5
   :crowd-silhouette 4 :hatching 3 :gradient 3 :dot 2 :flat-white 1 :none 0})

(defn- panel-tone
  "Panel background tone = the most salient tone among the panel's lines, its
  authored :tone, and the focal character's archetype tone (nil-safe)."
  [patterns char->archetype panel lines]
  (let [foc  (first (:characters panel))
        cand (->> (concat (map :tone lines)
                          [(:tone panel)
                           (get-in patterns [:archetypes
                                             (resolve-archetype char->archetype foc patterns) :tone])])
                  (filter tones))]
    (when (seq cand)
      (apply max-key #(get tone-salience % 0) cand))))

(defn analyze-panel
  "Enrich a storyboard panel. Analyzes each `:dialogue` line (and `:narration` /
  `:sfx` when present), then sets the panel's dominant `:tone` and any `:fx`."
  [patterns char->archetype panel]
  (let [dlg   (mapv #(analyze-line patterns char->archetype %) (:dialogue panel))
        narr  (when-let [n (:narration panel)]
                (analyze-line patterns char->archetype
                              (if (map? n) (assoc n :kind :narration) {:kind :narration :text n})))
        sfx   (mapv #(analyze-line patterns char->archetype (assoc % :kind :sfx)) (:sfx panel))
        lines (into (vec (concat dlg (when narr [narr]))) sfx)
        tone  (panel-tone patterns char->archetype panel lines)
        fx    (vec (distinct (mapcat :fx lines)))]
    (cond-> (assoc panel :dialogue dlg)
      narr      (assoc :narration (:text narr) :narration-style (dissoc narr :text))
      (seq sfx) (assoc :sfx sfx)
      tone      (assoc :tone tone)
      (seq fx)  (assoc :fx fx))))

(defn analyze-page
  "Enrich every panel of a page/storyboard. `page` = {:panels [...]}; `cast` maps
  speaker → archetype keyword (e.g. {\"ガター\" :hot-blooded})."
  ([patterns page] (analyze-page patterns {} page))
  ([patterns cast page]
   (update page :panels (fn [ps] (mapv #(analyze-panel patterns cast %) ps)))))
