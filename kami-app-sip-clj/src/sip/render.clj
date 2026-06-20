(ns sip.render
  "Compose per-panel render prompts from the anchor bible + storyboard panels,
  drive the local image-gen engine, and record everything as datoms.

  This is the Clojure + Datomic replacement for the old JSON-LD anchors + Python
  render scripts. Two facts learned from the 2026-06-18 render verification are
  baked in, not just documented:

    1. CLIP 77-token truncation. We drive image-gen's `/generate` directly (no
       server-side style prefix/suffix), so we own the whole 77-token window and
       compose STYLE-FIRST under a hard word budget — style/color never get cut.

    2. IP-Adapter on the diffusers app (AnimagineXL 4.0 + MPS float32) returns
       noise. Tag-only rendering is faithful, so we render tag-only here and keep
       `:sip.panel/refs` as metadata for the future ComfyUI IP-Adapter path."
  (:require [clojure.edn :as edn]
            [clojure.java.io :as io]
            [clojure.string :as str]
            [clojure.data.json :as json]
            [sip.store :as store]
            [sip.storyboard :as sb]
            [sip.lore :as lore])
  (:import [java.net URI]
           [java.util Base64]
           [java.net.http HttpClient HttpClient$Version HttpRequest HttpRequest$BodyPublishers
                          HttpResponse$BodyHandlers]))

;; ---------------------------------------------------------------------------
;; Anchors
;; ---------------------------------------------------------------------------

(defn anchors
  "Load the EDN anchor bible from the classpath (resources/render_anchors.edn)."
  []
  (with-open [r (io/reader (io/resource "render_anchors.edn"))]
    (edn/read (java.io.PushbackReader. r))))

;; ---------------------------------------------------------------------------
;; Composition (pure)
;; ---------------------------------------------------------------------------

(def ^:private dims
  "Aspect keyword → [w h], multiples of 8 (mirrors image-gen config ASPECT_RATIOS)."
  {:16x9 [1216 688] :9x16 [688 1216] :1x1 [1024 1024]
   :4x3 [1152 864] :3x4 [864 1152] :3x2 [1152 768] :2x3 [768 1152]})

(defn aspect->dims [aspect] (get dims aspect (dims :2x3)))

(def ^:private nei-light-cues
  ["ポッド" "光体" "発光" "半透明" "粒子" "光の" "明滅" "覚醒" "起動"
   "pod" "glow" "translucent" "luminous" "light field" "emergence" "awaken"])

(defn- nei-form
  "Pick Nei's embodied (:nei) vs light-figure (:nei-light) anchor from the panel's
  prose. Light-form for pod/awakening/abstract beats; embodied otherwise."
  [{:keys [description emotion colorNote location]}]
  (let [blob (str/lower-case (str description " " emotion " " colorNote " " location))]
    (if (some #(str/includes? blob (str/lower-case %)) nei-light-cues) :nei-light :nei)))

(defn- resolve-characters
  "Storyboard character keywords → anchor keywords, swapping nei→nei-light per cue."
  [panel]
  (let [form (nei-form panel)]
    (mapv (fn [c] (if (= c :nei) form c)) (:characters panel))))

(defn focal-character
  "パネル1キャラクター — one character per panel. The first dialogue speaker if
  they're in the cast (natural shot / reverse-shot), else the first listed.
  Applies the nei→nei-light swap for awakening/ghost-space beats. nil if none."
  [panel]
  (let [chars (:characters panel)
        sp    (some-> (first (:dialogue panel)) :speaker str str/lower-case keyword)
        pick  (or (some #{sp} chars) (first chars))]
    (when pick (if (= pick :nei) (nei-form panel) pick))))

(defn- framing
  "camera string → 1-2 booru framing tags."
  [camera]
  (let [c (str/lower-case (str camera))
        seg (str/trim (first (str/split c #"/")))
        shot (cond
               (str/includes? seg "extreme close") "extreme close-up"
               (str/includes? seg "close")         "close-up"
               (str/includes? seg "extreme wide")  "extreme wide shot"
               (str/includes? seg "wide")          "wide shot"
               (str/includes? seg "medium")        "medium shot"
               (str/includes? seg "over")          "over the shoulder"
               (str/includes? seg "two")           "two shot"
               :else "cinematic shot")
        angle (cond
                (str/includes? c "bird")  "from above"
                (str/includes? c "low")   "from below"
                (re-find #"over" c)       nil
                :else nil)]
    (filterv some? [shot angle])))

(def ^:private emotion->tag
  {"静寂" "serene" "静けさ" "serene" "目覚め" "awakening mood"
   "温もり" "warm mood" "温かさ" "warm mood" "やわらか" "tender"
   "戸惑い" "puzzled expression" "問い" "questioning expression"
   "歓び" "joyful expression" "喜び" "joyful expression" "受容" "gentle expression"
   "緊張" "tense" "接近" "intimate" "非分離" "intimate"
   "見守り" "watchful expression" "充足" "content expression"
   "発見" "wonder" "驚き" "surprised expression" "歓喜" "joyful expression"
   "ためらい" "hesitant expression" "余韻" "lingering quiet"})

(defn- mood-tags [emotion]
  (->> emotion->tag
       (keep (fn [[jp en]] (when (str/includes? (str emotion) jp) en)))
       distinct (take 2) vec))

(defn- env-key [location]
  (let [l (str location)]
    (cond
      (re-find #"事務所|office|デスク|オフィス" l) :schwa-office
      (re-find #"キッチン|アパート|kitchen|apartment|テーブル|窓際|室内" l) :tamaki-apartment
      (re-find #"遊歩道|並木|沿い|walkway|path|道" l) :canal-path
      (re-find #"運河|水の都|canal|水面" l) :water-city
      :else :water-city)))

(defn- subject-count
  "Combine present character subjects into a booru subject phrase."
  [present chars-anchors]
  (let [subs (keep #(get-in chars-anchors [% :subject]) present)
        g (count (filter #{"1girl"} subs))
        b (count (filter #{"1boy"} subs))]
    (->> [(case g 0 nil 1 "1girl" 2 "2girls" (str g "girls"))
          (case b 0 nil 1 "1boy" (str b "boys"))]
         (filterv some?))))

(defn- word-count [tags] (count (str/split (str/join " " tags) #"\s+")))

(defn- take-budget
  "Greedily concat tag groups, never exceeding `budget` words. Earlier groups win."
  [budget groups]
  (reduce (fn [acc tag]
            (let [acc' (conj acc tag)]
              (if (> (word-count acc') budget) (reduced acc) acc')))
          [] (distinct (apply concat groups))))

(defn compose
  "Panel map + anchors → render spec {:tags :prompt :neg :refs :aspect :dims}.
  STYLE-FIRST + word-budgeted so it survives CLIP 77 tokens."
  [{:keys [anchors panel]}]
  (let [{:keys [style-lead quality-tail word-budget base-negative
                volume-color characters environments aspect-by-layout]} anchors
        present  (if-let [f (focal-character panel)] [f] []) ; one character per panel
        char-an  characters
        col      (get volume-color (:area panel) [])
        env      (get-in environments [(env-key (:location panel)) :tags] [])
        subj     (subject-count present char-an)
        ;; cap identity tags per character so every present character keeps its
        ;; most-distinguishing tags in a two/three-shot (else the first character
        ;; eats the budget and the rest render generic).
        per-char (if (> (count present) 1) 4 7)
        id-tags  (vec (mapcat #(take per-char (get-in char-an [% :tags] [])) present))
        ;; priority groups (high → low); style & color lead, env trails
        body     (take-budget word-budget
                              [style-lead (vec (take 2 col)) (framing (:camera panel))
                               subj id-tags (mood-tags (:emotion panel))
                               (vec (take 3 env)) quality-tail])
        neg      (->> (concat base-negative (mapcat #(get-in char-an [% :negative] []) present))
                      distinct vec)
        refs     (->> present (keep #(get-in char-an [% :ref])) vec)
        aspect   (get aspect-by-layout (:layout panel) :2x3)]
    {:tags body
     :prompt (str/join ", " body)
     :neg neg
     :refs refs
     :aspect aspect
     :dims (aspect->dims aspect)}))

;; ---------------------------------------------------------------------------
;; image-gen HTTP client — /generate (we own the full prompt, tag-only)
;; ---------------------------------------------------------------------------

(def ^:private b64 (Base64/getDecoder))

(defn- post-json [^HttpClient client url body]
  (let [req (-> (HttpRequest/newBuilder (URI/create url))
                (.header "content-type" "application/json")
                (.timeout (java.time.Duration/ofMinutes 40))
                (.POST (HttpRequest$BodyPublishers/ofString (json/write-str body)))
                (.build))
        resp (.send client req (HttpResponse$BodyHandlers/ofString))]
    (if (<= 200 (.statusCode resp) 299)
      (json/read-str (.body resp) :key-fn keyword)
      (throw (ex-info "image-gen error" {:status (.statusCode resp) :body (.body resp)})))))

(defn render!
  "Render `spec` via image-gen /generate and write the PNG to `out-path`.
  Returns {:path :seed :ms}. `base` defaults to $IMAGEGEN_URL or :8100."
  [spec out-path & {:keys [base seed steps]
                    :or {base (or (System/getenv "IMAGEGEN_URL") "http://localhost:8100")
                         steps 28}}]
  (let [[w h] (:dims spec)
        ;; force HTTP/1.1 — uvicorn rejects the JDK client's default h2c upgrade
        client (-> (HttpClient/newBuilder) (.version HttpClient$Version/HTTP_1_1) (.build))
        res (post-json client (str base "/generate")
                       {:prompt (:prompt spec)
                        :negative_prompt (str/join ", " (:neg spec))
                        :width w :height h
                        :num_inference_steps steps
                        :seed seed})
        ;; /generate returns a data-URL ("data:image/png;base64,XXXX"); strip the prefix
        raw   (:image_base64 res)
        b64s  (if-let [i (str/index-of raw ",")] (subs raw (inc i)) raw)
        bytes (.decode b64 ^String b64s)]
    (io/make-parents (io/file out-path))
    (with-open [o (io/output-stream out-path)] (.write o bytes))
    {:path out-path :seed (:seed res) :ms (:generation_time_ms res)}))

;; ---------------------------------------------------------------------------
;; Datoms — anchors + panels into the world (datalevin)
;; ---------------------------------------------------------------------------

(defn anchors-tx
  "EDN anchor bible → :sip.anchor/* datoms (characters + environments)."
  [an]
  (concat
   (for [[id {:keys [name tags negative ref]}] (:characters an)]
     (cond-> {:sip.anchor/id id :sip.anchor/kind :character
              :sip.anchor/tags (vec tags) :sip.anchor/negative (vec negative)}
       name (assoc :sip.anchor/name name)
       ref  (assoc :sip.anchor/ref ref)))
   (for [[id {:keys [tags]}] (:environments an)]
     {:sip.anchor/id id :sip.anchor/kind :environment :sip.anchor/tags (vec tags)})))

(defn panel-tx
  "One storyboard panel + its composed spec → a :sip.panel/* datom map."
  [an panel]
  (let [{:keys [tags prompt neg refs aspect]} (compose {:anchors an :panel panel})]
    (cond-> {:sip.panel/id (:id panel)
             :sip.panel/area [:sip.area/id (:area panel)]
             :sip.panel/chapter (long (:chapter panel))
             :sip.panel/aspect aspect
             :sip.panel/prompt prompt
             :sip.panel/tags tags
             :sip.panel/neg neg
             :sip.panel/characters (mapv keyword (:characters panel))}
      (:page panel)        (assoc :sip.panel/page (:page panel))
      (:layout panel)      (assoc :sip.panel/layout (:layout panel))
      (:camera panel)      (assoc :sip.panel/camera (:camera panel))
      (:location panel)    (assoc :sip.panel/location (:location panel))
      (:description panel) (assoc :sip.panel/description (:description panel))
      (:emotion panel)     (assoc :sip.panel/emotion (:emotion panel))
      (seq refs)           (assoc :sip.panel/refs refs))))

(defn- areas-tx
  "The 8 learning-areas (= story volumes) as :sip.area datoms, so panels can
  resolve their :sip.panel/area lookup-ref. Mirrors `sip.world/game-tx`."
  []
  (for [{:keys [id title volume season theme motifs]} (lore/volumes)]
    {:sip.area/id id :sip.area/title title :sip.area/volume volume
     :sip.area/season season :sip.area/theme theme
     :sip.area/motif (vec motifs) :sip.area/open? (= volume 1)}))

(defn load!
  "Connect the world at `dir`, transact areas + anchors + all storyboard panels
  (each with its composed prompt). Returns {:areas n :anchors n :panels n}."
  [dir]
  (let [an   (anchors)
        conn (store/connect dir)
        ps   (sb/panels)]
    (store/transact! conn (vec (areas-tx)))
    (store/transact! conn (vec (anchors-tx an)))
    (store/transact! conn (mapv #(panel-tx an %) ps))
    {:areas (count (areas-tx)) :anchors (count (:characters an)) :panels (count ps)}))

;; ---------------------------------------------------------------------------
;; Batch render + provenance (render outputs are datoms, never erased)
;; ---------------------------------------------------------------------------

(defn render-one!
  "Compose + render panel `p` to `out-dir/<id>.png`, then record a
  `:sip.render/*` provenance datom against the panel. Returns the render map."
  [conn an p out-dir & {:keys [seed steps] :or {seed 4242 steps 28}}]
  (let [spec (compose {:anchors an :panel p})
        out  (str out-dir "/" (:id p) ".png")
        r    (render! spec out :seed seed :steps steps)]
    (store/transact! conn
      [{:sip.render/panel  [:sip.panel/id (:id p)]
        :sip.render/path   (:path r)
        :sip.render/seed   (long (or (:seed r) seed))
        :sip.render/ms     (long (or (:ms r) 0))
        :sip.render/engine (str "image-gen " (:model an "animagine-xl-4.0"))
        :sip.render/prompt (:prompt spec)}])
    (assoc r :id (:id p))))

(defn render-all!
  "Connect the world at `dir`, ensure anchors+panels are loaded, then render every
  panel to `out-dir` recording provenance. `:only` limits to a panel-id prefix
  (e.g. \"02-\" for one chapter); `:limit` caps the count. Returns a summary."
  [dir out-dir & {:keys [only limit seed steps]}]
  (let [an   (anchors)
        conn (store/connect dir)
        ps   (cond->> (sb/panels)
               only  (filter #(str/starts-with? (str (:id %)) only))
               limit (take limit))]
    (println "rendering" (count ps) "panel(s) →" out-dir)
    (let [done (doall
                (for [p ps]
                  (try
                    (let [r (render-one! conn an p out-dir
                                         :seed (or seed 4242) :steps (or steps 28))]
                      (println "  ✓" (:id p) "→" (:path r) (str "(" (:ms r) "ms)"))
                      r)
                    (catch Exception e
                      (println "  ✗" (:id p) (.getMessage e)) nil))))]
      {:rendered (count (filter some? done)) :total (count ps) :out out-dir})))

;; ---------------------------------------------------------------------------
;; CLI
;; ---------------------------------------------------------------------------

(defn -main
  "  compose <panel-id>            — print the composed prompt (no server, no DB)
   render  <panel-id> [out.png]  — compose + render via image-gen /generate
   load    [dir]                 — transact anchors + all panels into datalevin"
  [& [cmd a b]]
  (case cmd
    "compose"
    (let [an (anchors) p (sb/panel-by-id a) spec (compose {:anchors an :panel p})]
      (println "panel  " a "→ focal" (focal-character p) "aspect" (:aspect spec) (:dims spec))
      (println "words  " (count (str/split (:prompt spec) #"\s+")))
      (println "prompt " (:prompt spec))
      (println "neg    " (str/join ", " (take 8 (:neg spec)) ) "…"))

    "render"
    (let [an (anchors) p (sb/panel-by-id a)
          spec (compose {:anchors an :panel p})
          out (or b (str (System/getProperty "java.io.tmpdir") "/sip-render-" a ".png"))]
      (println "rendering" a "→" out)
      (println "prompt:" (:prompt spec))
      (let [r (render! spec out :seed 4242)]
        (println "done:" (:path r) "seed" (:seed r) (:ms r) "ms")))

    "load"
    (let [dir (or a (str (System/getProperty "java.io.tmpdir") "/sip-world"))]
      (println "loading anchors + panels into" dir)
      (println (load! dir)))

    "render-all"
    ;; render-all [only-prefix] [out-dir] — batch render + record provenance
    (let [out (or b (str (or (System/getenv "SIP_IP_ROOT") "../../260208-spirit-in-physics")
                         "/resources/images/sip-render"))
          dir (str (System/getProperty "java.io.tmpdir") "/sip-world")]
      (println (render-all! dir out :only (when (and a (not= a "all")) a))))

    (println "usage: compose <id> | render <id> [out] | render-all [prefix] [dir] | load [dir]"))
  (flush))
