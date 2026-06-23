(ns kami.backend.browser
  "L1 backend (cljs) — drives the `kami-clj-host` WASM module → `kami-render` →
  WebGPU in the browser. The real GPU path (ARCHITECTURE.md §4/§9). Implements
  `kami.gpu/IGpuBackend` by calling the `KamiCljHost` wasm-bindgen exports
  (register_mesh/register_material/register_shader/submit_frame/resize), which
  decode the KAMI columnar buffer (`kami.ipc/pack`) and render one instanced pass.

  `kami-clj-host` is the Rust crate `../../kami-clj-host` built with
  `wasm-pack build --target web --features host`; its JS glue exposes
  `KamiCljHost.create(canvas) -> Promise<host>`."
  (:require [kami.gpu :as gpu]
            [cljs.core.async :refer [chan put!]]))

(defn- ->u8 [buffer]
  "Convert the packed :buffer (a vector of 0-255 ints) to a Uint8Array for the
  wasm boundary. `kami.ipc/pack` already produced GPU-aligned bytes."
  (js/Uint8Array. (into-array buffer)))

(defn- ->f32 [xs] (js/Float32Array. (into-array xs)))
(defn- ->u32 [xs] (js/Uint32Array. (into-array xs)))

;; A thin record wrapping the wasm `KamiCljHost` instance.
(defrecord BrowserBackend [host]
  gpu/IGpuBackend
  ;; `^js` on the host hints Closure not to munge the wasm-bindgen method names
  ;; under :advanced (without it, register_mesh/submit_frame get renamed → the
  ;; "this.host.xd is not a function" failure on a real GPU run).
  (register-mesh! [_ id vertices indices]
    (.register_mesh ^js host id (->f32 vertices) (->u32 indices)))
  (register-material! [_ id params]
    (.register_material ^js host id (->f32 (or params []))))
  (register-shader! [_ id wgsl layout]
    (.register_shader ^js host id wgsl (or layout "")))
  (submit-frame! [_ packed]
    ;; packed = {:buffer :len :meta …}; meta travels as JSON, buffer as bytes.
    (.submit_frame ^js host
                   (js/JSON.stringify (clj->js (:meta packed)))
                   (->u8 (:buffer packed))))
  (resize! [_ w h]
    (.resize ^js host w h)))

(defn make
  "Create a browser GPU backend bound to canvas id `:canvas`. Returns a channel
  that yields the backend once `KamiCljHost.create` resolves (async adapter/device
  request). `:host-ctor` lets callers inject the wasm class (default
  `js/KamiCljHost`)."
  [{:keys [canvas host-ctor]}]
  ;; `KamiCljHost.create` returns a JS Promise; bridge it to the result channel.
  ;; (cljs `go` has no `await`; convert the promise with `.then` + `put!`.)
  (let [out  (chan)
        el   (.getElementById js/document canvas)
        ctor (or host-ctor js/KamiCljHost)]
    (-> (.create ctor el)
        (.then  (fn [host] (put! out (->BrowserBackend host))))
        (.catch (fn [err] (js/console.error "KamiCljHost.create failed" err))))
    out))
