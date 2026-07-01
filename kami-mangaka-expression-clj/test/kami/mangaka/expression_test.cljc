(ns kami.mangaka.expression-test
  (:require [clojure.test :refer [deftest is testing]]
            [kami.mangaka.expression :as e]))

(def P (e/load-patterns))

(deftest vocab-lockstep
  (testing "code vocab == edn :vocab (drift guard, manifest-matches-registry 流儀)"
    (let [v (:vocab P)]
      (is (= e/registers   (:register v)))
      (is (= e/bubbles     (:bubble v)))
      (is (= e/expressions (:expression v)))
      (is (= e/font-roles  (:font-role v)))
      (is (= e/weights     (:weight v)))
      (is (= e/tones       (:tone v)))
      (is (= e/postures    (:posture v)))
      (is (= e/marks       (:mark v)))))
  (testing "every archetype/register/cue references only known vocab"
    (doseq [[_ a] (:archetypes P)]
      (is (e/bubbles (:bubble a))) (is (e/tones (:tone a)))
      (is (e/font-roles (:font-role a))) (is ((set e/weights) (:weight a)))
      (is (e/expressions (:expression a))))
    (doseq [[_ r] (:registers P) :when (:bubble r)]
      (is (e/bubbles (:bubble r))))))

(deftest expression-of
  (is (= "Angry" (e/expression-of "rage")))
  (is (= "Angry" (e/expression-of "激昂")))
  (is (= "Happy" (e/expression-of "喜")))
  (is (= "Determined" (e/expression-of "Determined")) "already-canonical passes through")
  (is (= "Neutral" (e/expression-of "???")) "unknown → Neutral"))

(deftest weight-ops
  (is (= 2 (e/weight-index :regular)))
  (is (= :heavy (e/weight-bump :bold 1)))
  (is (= :heavy (e/weight-bump :heavy 3)) "clamped at :heavy")
  (is (= :faint (e/weight-bump :faint -2)) "clamped at :faint"))

(deftest resolve-style-registers
  (testing "strategist + speech = calm oval mincho, scale ~1"
    (let [s (e/resolve-style P {:archetype :strategist :register :speech})]
      (is (= :oval (:bubble s))) (is (= :mincho (:font-role s)))
      (is (= :regular (:weight s))) (is (== 1.0 (:scale s)))))
  (testing "hot-blooded + shout + high intensity = spike, heavy, big, flash"
    (let [s (e/resolve-style P {:archetype :hot-blooded :register :shout :intensity 1.0})]
      (is (= :spike (:bubble s))) (is (= :heavy (:weight s)))
      (is (> (:scale s) 2.0)) (is (= :flash (:tone s)))
      (is (some #{:speed-lines} (:fx s)))
      (is (some #{:anger-vein} (:marks s)))))
  (testing "mob + chatter = faint tiny handwritten on crowd silhouette (薄く小さい)"
    (let [s (e/resolve-style P {:archetype :mob :register :chatter})]
      (is (= :faint (:weight s))) (is (< (:scale s) 0.8))
      (is (= :cloud (:bubble s))) (is (= :crowd-silhouette (:tone s)))
      (is (= :handwritten (:font-role s)))))
  (testing "whisper = light + small (文字が薄い)"
    (let [s (e/resolve-style P {:archetype :strategist :register :whisper})]
      (is (= :light (:weight s))) (is (< (:scale s) 1.0))))
  (testing "low intensity attenuates dramatic tone to :dot"
    (let [s (e/resolve-style P {:archetype :hot-blooded :register :speech :intensity 0.1})]
      (is (= :dot (:tone s)) "focus-lines/flash は弱い感情では :dot に減衰")))
  (testing "nameplate is an inverted (white-on-black) gothic label"
    (let [s (e/resolve-style P {:register :nameplate})]
      (is (:invert s)) (is (= :gothic (:font-role s))) (is (= :square (:bubble s))))))

(deftest infer-register-test
  (is (= :shout (e/infer-register {:text "バカな！！"})))
  (is (= :shout (e/infer-register {:text "止まれ！"})))
  (is (= :whisper (e/infer-register {:text "（まずいな…）"})))
  (is (= :speech (e/infer-register {:text "そうですか。"})))
  (is (= :narration (e/infer-register {:kind :narration :text "その頃、王宮では。"}))))

(deftest analyze-line-test
  (let [cast {"ガター" :hot-blooded "ナイベイン" :strategist}]
    (testing "speaker archetype + inferred shout drives the bubble/weight/tone"
      (let [l (e/analyze-line P cast {:speaker "ガター" :text "バカな！！" :intensity 0.9})]
        (is (= :hot-blooded (:archetype l))) (is (= :shout (:register l)))
        (is (= :spike (:bubble l))) (is (= :heavy (:weight l)))
        (is (= :flash (:tone l)))))
    (testing "explicit bubble/weight win over the resolved style"
      (let [l (e/analyze-line P cast {:speaker "ガター" :text "バカな！！" :bubble :jagged :weight :regular})]
        (is (= :jagged (:bubble l))) (is (= :regular (:weight l)))))
    (testing "calm strategist speech stays oval/regular"
      (let [l (e/analyze-line P cast {:speaker "ナイベイン" :text "こちらは構わないが。"})]
        (is (= :oval (:bubble l))) (is (= :regular (:weight l)))))
    (testing "an explicit nil (adapter 'unset') does NOT clobber the resolved default"
      (let [l (e/analyze-line P cast {:speaker "ガター" :text "バカな！！"
                                      :bubble nil :register nil :expression nil :intensity 1.0})]
        (is (= :shout (:register l))) (is (= :spike (:bubble l)))
        (is (= "Angry" (:expression l)))))))

(deftest analyze-panel+page
  (let [cast {"ガター" :hot-blooded "スラッカ" :energetic}
        page {:panels [{:characters ["ガター"]
                        :dialogue [{:speaker "ガター" :text "バカな！！" :intensity 1.0}]}
                       {:characters ["スラッカ"]
                        :dialogue [{:speaker "スラッカ" :text "ちょっといいかな？" :register :shout}]}
                       {:dialogue [{:text "…" :register :chatter}
                                   {:text "…" :register :chatter}]}]}
        out (e/analyze-page P cast page)
        [p0 p1 p2] (:panels out)]
    (testing "angry shout panel gets a dramatic tone + speed-lines fx"
      (is (e/dramatic-tones (:tone p0)))
      (is (some #{:speed-lines} (:fx p0))))
    (testing "energetic interjection → burst/spike, radial-ish"
      (is (#{:spike :burst} (:bubble (first (:dialogue p1))))))
    (testing "crowd chatter panel = crowd silhouette tone, faint tiny text"
      (is (= :crowd-silhouette (:tone p2)))
      (is (every? #(= :faint (:weight %)) (:dialogue p2)))
      (is (every? #(< (:scale %) 0.8) (:dialogue p2))))))
