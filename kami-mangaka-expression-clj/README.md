# kami-mangaka-expression-clj

Work-agnostic **manga expression patterns** — 「キャラクターごとに、表情・姿勢・
吹き出しの形・文字の薄さ・大きさ・背景トーンを変える」という漫画的表現を **data から**
決める純粋レイヤ (ADR-2607012100, Tier-1 mangaka)。

出典は **HUNTER×HUNTER 王位継承戦編**（冨樫義博）のコマ観察
（`resources/mangaka_expression_patterns.edn` の `:source` / `:observations`）。
抽出した漫画文法:

- **モブのざわめき** — 薄い小さな手書き文字＋小さな雲形＋群衆シルエット背景 (`:chatter`)
- **キャラ名札** — 黒箱に白抜きゴシック (`:nameplate`)
- **割り込み/元気** — 挙手＋放射状集中線＋太く大きなトゲ/バースト吹き出し (`:energetic`+`:shout`)
- **激昂アップ** — 前傾＋歯剥き＋黒トゲ/フラッシュ背景＋汗・怒張 (`:hot-blooded`+`:shout`)
- **冷酷/威圧** — 直立＋重い影(暗ビネット)＋太明朝の決め台詞 (`:cold-menace`)
- **独白/地の文** — 明朝の囲み (`:monologue`/`:narration`)

## モデル

1 行分のスタイルは

```
archetype(キャラ類型) ← register(セリフ種別) ← expression-cue(感情) ← intensity(強度)
```

の順に merge して決まる（`resolve-style`）。出力は **semantic タグ**のみ:

| キー | 意味 | 語彙 |
|---|---|---|
| `:expression` | 表情 | `kami.mangaka.scene` の Rust Expression と一致 |
| `:posture` | 姿勢 | `:upright :lean-in :arm-raised :recoil :slump :point :cross-arms :turn-away` |
| `:bubble` | 吹き出しの形 | `kami.mangaka.text` ∪ `{:spike :burst}` |
| `:font-role` | 写植ロール | ai-gftd-mangaka の `manga-font-palette` (fontRole) |
| `:weight` | 文字の薄さ↔太さ | `[:faint :light :regular :bold :heavy]` |
| `:scale` | 文字の大きさ | 倍率 (double, `:scale-max` で clamp) |
| `:tone` | 背景トーン | `:focus-lines :radial-burst :flash :vignette-dark :crowd-silhouette …` |
| `:marks` / `:fx` | 描き文字マーク / 効果 | 汗・怒張・集中線 など |

見た目への写像（opacity/font-size/screentone）は **renderer 側**が持つ
（`kami-mangaka-text-clj` の hiccup+CSS / `kami-mangaka-page-clj` の Java2D）。
本モジュールは純 semantic — babashka-safe / JVM・cljs・WASM 可搬。

## API

```clojure
(require '[kami.mangaka.expression :as e])
(def P (e/load-patterns))                      ; clj: resources の edn を読む

(e/resolve-style P {:archetype :hot-blooded :register :shout :intensity 1.0})
;; => {:bubble :spike :weight :heavy :scale 2.4 :tone :flash :fx [:speed-lines]
;;     :marks [:anger-vein :sweat] :expression "Angry" :posture :lean-in …}

;; storyboard へ適用（cast: speaker → archetype）
(e/analyze-page P {"ガター" :hot-blooded "スラッカ" :energetic}
  {:panels [{:characters ["ガター"]
             :dialogue [{:speaker "ガター" :text "バカな！！" :intensity 1.0}]}]})
```

`register` 省略時は `infer-register` が本文から推定（`！！`→`:shout`、`（…）`→`:whisper`）。
明示された `:bubble` / `:weight` / `:scale` は常に解決結果に勝つ。

## 消費側

- **kami.mangaka.text / .page** — 解決済みの `:weight`/`:scale`/`:tone`/`:bubble` と
  `:chatter`/`:nameplate` register を hiccup+CSS / Java2D で描画。
- **ai-gftd-mangaka** — langgraph `ai.gftd.mangaka.analyzeExpression` が `analyze-page`
  を storyboard datom（`:panel/bubbles` 等）に橋渡しし、`:bubble/shape` `:bubble/font`
  `:bubble/font-size` `:panel/tone` を enrich する。
- **genko**（kami-engine-sdk）— fukidashi type / tone を同語彙で受ける。

## テスト

```bash
clojure -M:test    # vocab lock-step / resolve-style / analyze
bb test
```
