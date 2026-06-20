(ns sip.world
  "Authoring (JVM): build the water-city world as datoms, then emit the portable
  snapshot the browser boots from. `clojure -M:datomic:build` runs `-main`.

  The world = a Datomic/datalevin transaction. The render half (water, sakura,
  light, camera) becomes the scene snapshot served to `kami-render`; the game
  half (8 areas, the player, the still-unnamed Ghost Agent) lives in the same
  store as the source of truth for `sip.store`."
  (:require [sip.schema :as schema]
            [sip.store :as store]
            [sip.lore :as lore]
            [kami.db :as kdb]
            [clojure.java.io :as io]
            [clojure.edn :as edn]))

;; --- assets (procedural placeholders; real .kmesh/.kmat resolve by id) -------

(def assets
  [{:asset/id "mesh/water"   :asset/kind :mesh     :asset/inline (pr-str {:prim :plane :size 1})}
   {:asset/id "mesh/sakura"  :asset/kind :mesh     :asset/inline (pr-str {:prim :tree  :style :cherry})}
   {:asset/id "mesh/lantern" :asset/kind :mesh     :asset/inline (pr-str {:prim :lantern})}
   {:asset/id "mesh/house"   :asset/kind :mesh     :asset/inline (pr-str {:prim :cube})}
   {:asset/id "mat/water"    :asset/kind :material :asset/inline (pr-str {:albedo [0.55 0.74 0.86] :metallic 0.2 :roughness 0.15})}
   {:asset/id "mat/sakura"   :asset/kind :material :asset/inline (pr-str {:albedo [0.97 0.78 0.86] :roughness 0.8})}
   {:asset/id "mat/wood"     :asset/kind :material :asset/inline (pr-str {:albedo [0.80 0.64 0.46] :roughness 0.7})}
   {:asset/id "mat/warm"     :asset/kind :material :asset/inline (pr-str {:emissive [1.0 0.82 0.55]})}])

;; A unit water plane (±1) scaled to a 8×8 tile; a 5×5 grid (spacing 8) over
;; [-16,16] forms one contiguous canal surface — the heart of the water city.
(defn- canal-tile [[x z]]
  {:kami/eid (random-uuid) :kami/name "canal"
   :transform/translation [(double x) 0.0 (double z)]
   :transform/rotation [0.0 0.0 0.0 1.0]
   :transform/scale [4.0 1.0 4.0]
   :mesh/asset [:asset/id "mesh/water"] :material/asset [:asset/id "mat/water"]})

(defn- sakura [[x z]]
  {:kami/eid (random-uuid) :kami/name "sakura"
   :transform/translation [(double x) 0.0 (double z)]
   :transform/rotation [0.0 0.0 0.0 1.0]
   :transform/scale [1.3 1.4 1.3]
   :mesh/asset [:asset/id "mesh/sakura"] :material/asset [:asset/id "mat/sakura"]})

(defn- lantern [[x z]]
  {:kami/eid (random-uuid) :kami/name "lantern"
   :transform/translation [(double x) 1.1 (double z)]
   :transform/rotation [0.0 0.0 0.0 1.0]
   :transform/scale [0.5 0.9 0.5]
   :mesh/asset [:asset/id "mesh/lantern"] :material/asset [:asset/id "mat/warm"]})

(defn- house [[x z] h]
  {:kami/eid (random-uuid) :kami/name "house"
   :transform/translation [(double x) (double (/ h 2.0)) (double z)]
   :transform/rotation [0.0 0.0 0.0 1.0]
   :transform/scale [2.2 (double h) 2.0]
   :mesh/asset [:asset/id "mesh/house"] :material/asset [:asset/id "mat/wood"]})

(defn scene-tx
  "Render half: a contiguous canal plaza framed by cherry trees, waterside
  lanterns and wood townhouses — a calm water-city vignette under a warm key
  light, viewed from a gently downward camera."
  []
  (concat
   assets
   [;; camera — above and pitched ~18° down so the canal surface reads
    {:kami/eid (random-uuid) :kami/name "main-cam"
     :camera/fov 55.0 :camera/near 0.1 :camera/far 2000.0 :camera/active? true
     :transform/translation [0.0 9.0 22.0]
     :transform/rotation [-0.156 0.0 0.0 0.988]}    ; pitch down about +X
    {:kami/eid (random-uuid) :kami/name "morning-light"
     :light/kind :dir :light/color [1.0 0.93 0.84] :light/intensity 1.3
     :transform/rotation [0.0 0.0 0.0 1.0]}]
   ;; contiguous canal: 5×5 tiles over [-16,16]
   (map canal-tile (for [x (range -16 17 8) z (range -16 17 8)] [x z]))
   ;; cherry trees lining the two long edges of the canal
   (map sakura (concat (for [x (range -16 17 8)] [x -19])
                       (for [x (range -16 17 8)] [x  19])))
   ;; warm lanterns along the waterway
   (map lantern [[-12 -6] [-4 6] [4 -6] [12 6] [0 0]])
   ;; townhouses set back behind the cherry rows (varied heights)
   (map house [[-18 -24] [-9 -25] [0 -24] [9 -25] [18 -24]]
              [3.0 4.0 2.5 3.5 3.0])))

(defn game-tx
  "Game half: the 8 learning-areas (from the story-bible), the player, and their
  Ghost Agent — unnamed, awakening at stage 0. Asking its name is move one."
  []
  (let [areas (for [{:keys [id title volume season theme motifs]} (lore/volumes)]
                {:sip.area/id id :sip.area/title title :sip.area/volume volume
                 :sip.area/season season :sip.area/theme theme
                 :sip.area/motif (vec motifs)
                 :sip.area/open? (= volume 1)}) ; only Vol.1 (Water City) open at start
        agent {:sip.agent/named? false :sip.agent/bond 0
               :sip.agent/awakening 0 :sip.agent/voice :system-log}]
    (concat areas
            [agent
             {:sip.player/id (random-uuid)
              :sip.player/name "見習い"
              :sip.player/area [:sip.area/id :vol01-water-city]
              :sip.kokoro/value 0.6 :sip.kokoro/tempo 60.0}])))

(def water-city-env
  "Dawn over the canals — soft lavender sky, light fog on the water."
  {:clear [0.96 0.93 0.99 1.0]
   :sky   :dawn
   :fog   {:color [0.93 0.92 0.97] :density 0.015}})

(defn build!
  "Transact schema + world into a fresh store at `dir`; return the render
  snapshot (game state stays in the store for `sip.store`)."
  [dir]
  (let [conn (store/connect dir)]
    ;; datalevin's transact! returns the TxReport directly (not a future).
    (store/transact! conn (vec (scene-tx)))
    (store/transact! conn (vec (game-tx)))
    (kdb/snapshot (store/db conn)
                  {:scene "spirit-in-physics/water-city"
                   :env (pr-str water-city-env)})))

(defn -main
  "Build the snapshot and write it where the browser bundle can fetch it
  (public/snapshot.edn). Title: Spirit in Physics → https://sip.etzhayyim.com"
  [& [dir]]
  (let [dir (or dir (str (System/getProperty "java.io.tmpdir") "/sip-world"))
        snap (build! dir)
        out  (io/file "public" "snapshot.edn")]
    (io/make-parents out)
    (spit out (pr-str snap))
    (println "Spirit in Physics — wrote" (.getPath out)
             "(" (count (:snapshot/entities snap)) "entities,"
             (count (:snapshot/assets snap)) "assets )")
    ;; sanity: the snapshot the browser will load must be structurally valid
    (require 'kami.scene)
    (let [valid? (resolve 'kami.scene/valid?)]
      (when valid? (valid? snap)))
    (System/exit 0)))
