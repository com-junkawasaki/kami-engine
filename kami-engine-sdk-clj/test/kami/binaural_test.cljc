(ns kami.binaural-test
  "GPU-free, mixer-free contract tests for kami.binaural: the spatialization IR
  (ITD / ILD / distance / gains) and per-backend lowering. Pins the data
  contract long before any audio device is wired up."
  (:require [clojure.test :refer [deftest testing is]]
            [kami.binaural :as b]))

(defn- close? [a x] (< (Math/abs (- a x)) 1e-6))

;; default listener: at origin, facing -Z, up +Y → +X is to the right.

(deftest validation
  (testing "well-formed scene validates"
    (is (true? (b/valid? {:binaural/sources [{:id :a :cue :x :pos [1.0 0.0 0.0]}]}))))
  (testing "unknown rolloff kind rejected"
    (is (thrown? #?(:clj Exception :cljs js/Error)
                 (b/valid? {:binaural/rolloff {:kind :bogus}
                            :binaural/sources []}))))
  (testing "source without 3-vector :pos rejected"
    (is (thrown? #?(:clj Exception :cljs js/Error)
                 (b/valid? {:binaural/sources [{:id :a :pos [1.0 2.0]}]})))))

(deftest source-on-the-right
  ;; +X is to the right of the default listener → right ear leads & is louder.
  (let [{:keys [spatial]} (-> {:binaural/sources [{:id :r :cue :ping :pos [5.0 0.0 0.0]}]}
                              b/mix :binaural/sources first)]
    (testing "positive ITD (right ear leads) → left ear delayed"
      (is (pos? (:itd-s spatial)))
      (is (pos? (:delay-l-s spatial)))
      (is (close? (:delay-r-s spatial) 0.0)))
    (testing "positive ILD → right louder than left"
      (is (pos? (:ild-db spatial)))
      (is (> (:gain-r spatial) (:gain-l spatial))))
    (testing "azimuth ~ +90° to the right"
      (is (close? (:azimuth spatial) (/ Math/PI 2.0))))))

(deftest source-on-the-left-mirrors-right
  (let [r (-> {:binaural/sources [{:id :r :pos [5.0 0.0 0.0]}]} b/mix :binaural/sources first :spatial)
        l (-> {:binaural/sources [{:id :l :pos [-5.0 0.0 0.0]}]} b/mix :binaural/sources first :spatial)]
    (testing "left source mirrors the right one"
      (is (close? (:itd-s l) (- (:itd-s r))))
      (is (close? (:gain-l l) (:gain-r r)))
      (is (close? (:gain-r l) (:gain-l r)))
      (is (close? (:delay-r-s l) (:delay-l-s r))))))

(deftest dead-ahead-is-centered
  (let [s (-> {:binaural/sources [{:id :f :pos [0.0 0.0 -5.0]}]} b/mix :binaural/sources first :spatial)]
    (testing "source dead ahead → zero ITD/ILD, equal gains"
      (is (close? (:itd-s s) 0.0))
      (is (close? (:ild-db s) 0.0))
      (is (close? (:gain-l s) (:gain-r s)))
      (is (close? (:azimuth s) 0.0)))))

(deftest distance-rolloff-monotonic
  (let [g (fn [d kind] (b/distance-gain {:kind kind :ref 1.0 :max 100.0 :factor 1.0} d))]
    (testing "all rolloff laws are 1.0 at the reference distance"
      (is (close? (g 1.0 :inverse) 1.0))
      (is (close? (g 1.0 :linear) 1.0))
      (is (close? (g 1.0 :exponential) 1.0)))
    (testing "gain decreases with distance"
      (is (> (g 1.0 :inverse) (g 10.0 :inverse) (g 50.0 :inverse)))
      (is (> (g 1.0 :linear)  (g 10.0 :linear)))
      (is (> (g 1.0 :exponential) (g 10.0 :exponential))))
    (testing ":none keeps full gain"
      (is (close? (g 99.0 :none) 1.0)))))

(deftest itd-within-physical-bound
  ;; max ITD for a 0.0875 m head ≈ a/c·(π/2+1) ≈ 0.66 ms — never exceed ~0.7 ms.
  (let [s (-> {:binaural/sources [{:id :r :pos [100.0 0.0 0.0]}]} b/mix :binaural/sources first :spatial)]
    (is (< (:itd-s s) 7.0e-4))
    (is (> (:itd-s s) 5.0e-4))))

(deftest mix-preserves-order-and-ids
  (let [ir (b/mix {:binaural/sources [{:id :a :cue :x :pos [1.0 0.0 0.0]}
                                      {:id :b :cue :y :pos [0.0 0.0 -1.0]}]})]
    (is (= [:a :b] (map :source/id (:binaural/sources ir))))
    (is (= [:x :y] (map :source/cue (:binaural/sources ir))))))

(deftest emit-web-audio
  (let [ir  (b/mix {:binaural/sources [{:id :r :cue :ping :pos [5.0 0.0 0.0]}]})
        out (b/emit :web-audio ir)
        n   (-> out :nodes first)]
    (is (= :web-audio (:backend out)))
    (is (= :r (:id n)))
    (is (pos? (:delay-l n)))                ; right source → left delayed
    (is (> (:gain-r n) (:gain-l n)))
    (is (close? (:pan n) 1.0))))            ; sin(+90°) = +1 (full right)

(deftest emit-native-sample-delays
  (let [ir  (b/mix {:binaural/sources [{:id :r :cue :ping :pos [5.0 0.0 0.0]}]})
        out (b/emit :native ir {:sample-rate 48000})
        v   (-> out :voices first)]
    (is (= :native (:backend out)))
    (is (= 48000 (:sample-rate out)))
    (is (pos? (:delay-l-samples v)))        ; integer ITD in samples
    (is (zero? (:delay-r-samples v)))
    (is (> (:right-vol v) (:left-vol v)))))

(deftest emit-unknown-backend-passes-ir-through
  (let [ir (b/mix {:binaural/sources [{:id :a :pos [1.0 0.0 0.0]}]})]
    (is (= ir (:ir (b/emit :ps5-mixer ir))))))
