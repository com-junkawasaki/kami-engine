;; wit_test.clj — codegen + consistency gate for the kami:engine interface.
;;
;; Reads wit/kami-interface.edn (the ONE source), generates WIT, and asserts the generated WIT is
;; ABI-equivalent to the committed wit/kami-game/world.wit — same interfaces, same functions, same
;; lowered parameter/return types. If the EDN and the hand-written WIT drift, this throws.
;;
;;   bb scripts/wit_test.clj           # check (throws on drift)
;;   bb scripts/wit_test.clj --gen     # print the regenerated WIT
(require '[clojure.edn :as edn]
         '[clojure.string :as str])

(def idl   (edn/read-string (slurp "wit/kami-interface.edn")))
(def world (slurp "wit/kami-game/world.wit"))

;; ── semantic type → WASM ABI (the lowering the host/guest agree on) ──────────────────────
(defn wit-type [t] (case t (:eid :i64) "s64" (:f32 :i32) "s32"))

(defn- param-strs [[nm t]]
  (if (= t :str)
    [(str (name nm) "-ptr: s32") (str (name nm) "-len: s32")]   ;; a string lowers to (ptr, len)
    [(str (name nm) ": " (wit-type t))]))

(defn wit-func [fname {:keys [params ret]}]
  (str (name fname) ": func(" (str/join ", " (mapcat param-strs params)) ")"
       (when (and ret (not= ret :unit)) (str " -> " (wit-type ret))) ";"))

(defn gen-wit
  "Regenerate the full world.wit text from the EDN IDL."
  [idl]
  (str "package " (:package idl) ";\n\n"
       (str/join "\n\n"
         (for [[iname ispec] (:interfaces idl)]
           (str "// " (:doc ispec) "\ninterface " (name iname) " {\n"
                (str/join "\n" (for [[fn spec] (:funcs ispec)] (str "    " (wit-func fn spec))))
                "\n}")))
       "\n\nworld " (:world idl) " {\n"
       (str/join "\n" (for [[iname _] (:interfaces idl)]
                        (str "    import " (str/replace (:package idl) "@" (str "/" (name iname) "@")) ";")))
       "\n    export memory;\n"
       (str/join "\n" (for [[en spec] (:exports idl)] (str "    export " (wit-func en spec))))
       "\n}\n"))

;; ── canonical ABI signature: "iface.fn(types…)ret" — name/whitespace independent ─────────
(defn gen-canon [idl]
  (set (for [[iname ispec] (:interfaces idl), [fname fspec] (:funcs ispec)]
         (str (name iname) "." (name fname) "("
              (str/join "," (mapcat (fn [[_ t]] (if (= t :str) ["s32" "s32"] [(wit-type t)])) (:params fspec)))
              ")" (when (and (:ret fspec) (not= (:ret fspec) :unit)) (wit-type (:ret fspec)))))))

(defn wit-canon [wit]
  (let [nc (str/replace wit #"//[^\n]*" "")]
    (set (mapcat (fn [[_ iname body]]
                   (map (fn [[_ fname params ret]]
                          (str iname "." fname "("
                               (str/join "," (map second (re-seq #":\s*(s\d+)" params)))
                               ")" (when ret (second (re-find #"(s\d+)" ret)))))
                        (re-seq #"([\w-]+)\s*:\s*func\s*\(([^)]*)\)\s*(->\s*s\d+)?\s*;" body)))
                 (re-seq #"interface\s+([\w-]+)\s*\{([^}]*)\}" nc)))))

;; ── 3rd source: the kami-clj Builtin→HostImport map (ast.rs). Names only (its signatures live in
;;    a separate table); a HostImport "SceneGetX" derives interface.fn "scene.get-x". ─────────────
(def iface-prefixes ["Scene" "Physics" "Input" "Render" "Audio" "Time" "Random"])
(defn- kebab [s] (-> s (str/replace #"(.)([A-Z])" "$1-$2") str/lower-case))
(defn builtin-names [src]
  (set (for [[_ hi] (re-seq #"Some\(HostImport::(\w+)\)" src)
             :let [pfx (first (filter #(str/starts-with? hi %) iface-prefixes))]
             :when pfx]
         (str (str/lower-case pfx) "." (kebab (subs hi (count pfx)))))))
(defn idl-names [idl]
  (set (for [[iname ispec] (:interfaces idl), [fname _] (:funcs ispec)]
         (str (name iname) "." (name fname)))))

;; ── run ──────────────────────────────────────────────────────────────────────────────────
(if (some #{"--gen"} *command-line-args*)
  (println (gen-wit idl))
  (let [g (gen-canon idl), w (wit-canon world)
        edn-only (sort (clojure.set/difference g w))
        wit-only (sort (clojure.set/difference w g))
        ast (slurp "kami-engine-clj/src/ast.rs")
        bi  (builtin-names ast), in (idl-names idl)
        bi-edn (sort (clojure.set/difference in bi))    ;; in EDN, missing a Builtin host-import
        bi-only (sort (clojure.set/difference bi in))]  ;; a Builtin host-import the IDL doesn't list
    (println "── kami:engine interface — single-source consistency ──")
    (println (format "  EDN IDL: %d host functions across %d interfaces" (count g) (count (:interfaces idl))))
    (println (format "  WIT:     %d  ·  kami-clj Builtin host-imports: %d" (count w) (count bi)))
    (when (seq edn-only) (println "  WIT drift — only in EDN:" (vec edn-only)))
    (when (seq wit-only) (println "  WIT drift — only in WIT:" (vec wit-only)))
    (when (seq bi-edn)   (println "  Builtin drift — EDN fn with no host-import:" (vec bi-edn)))
    (when (seq bi-only)  (println "  Builtin drift — host-import not in EDN:" (vec bi-only)))
    (if (and (= g w) (= in bi))
      (println "  ✓ EDN IDL, world.wit, and the kami-clj Builtin map all agree — one source, three targets.")
      (throw (ex-info "kami:engine interface DRIFT"
                      {:wit-only wit-only :edn-only edn-only :builtin-only bi-only :edn-no-builtin bi-edn})))))
