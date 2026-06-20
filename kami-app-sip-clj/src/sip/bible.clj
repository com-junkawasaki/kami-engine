(ns sip.bible
  "Read the EDN story bible — the single source of truth, now Clojure data (not
  JSON-LD). Source: ../../260208-spirit-in-physics/story-bible/story-bible.edn
  (override the repo root with $SIP_IP_ROOT). JVM authoring helper."
  (:require [clojure.edn :as edn]
            [clojure.java.io :as io]))

(def ip-root (or (System/getenv "SIP_IP_ROOT") "../../260208-spirit-in-physics"))

(defn load-bible
  "The whole bible as a Clojure map, or nil if absent."
  []
  (let [f (io/file ip-root "story-bible/story-bible.edn")]
    (when (.exists f) (edn/read-string (slurp f)))))

(defn volumes
  "The 8 volumes (bible :hasPart)."
  [] (:hasPart (load-bible)))

(defn philosophy [] (:gh/corePhilosophy (load-bible)))
(defn nei-arc    [] (:gh/neiArc (load-bible)))
(defn havah-arc  [] (:gh/havahArc (load-bible)))
(defn sins       [] (:gh/threeOriginalSins (load-bible)))
