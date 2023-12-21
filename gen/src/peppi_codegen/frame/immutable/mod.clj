(ns peppi-codegen.frame.immutable.mod
  (:require
   [clojure.java.io :as io]
   [peppi-codegen.common :refer :all]
   [peppi-codegen.frame.common :refer :all]))

(defn array-type
  [ty]
  (cond
    (primitive-types ty) ["PrimitiveArray" ty]
    (nil? ty)            "NullArray"
    :else                ty))

(defn struct-field
  [{nm :name, ty :type, ver :version}]
  [nm (cond->> (array-type ty)
        ver (conj ["Option"]))])

(defn transpose-one-field-init
  [{idx :index, nm :name, ty :type, ver :version}]
  (let [real-target [:field-get "self" (or nm idx)]
        target (if ver "x" real-target)
        value (if (primitive-types ty)
                [:subscript [:method-call target "values"] "i"]
                [:method-call target "transpose_one" ["i" "version"]])]
    (if ver
      (wrap-map (as-ref real-target) "x" value)
      value)))

(defn transpose-one-fn
  [nm fields]
  (let [ctype (list "transpose" nm)]
    [:fn
     {:visibility "pub"
      :ret ctype}
     "transpose_one"
     [["&self"]
      ["i" "usize"]
      ["version" "Version"]]
     [:block
      [:struct-init ctype (->> fields
                               (filterv :type)
                               (mapv (juxt :name transpose-one-field-init)))]]]))

(defn into-immutable
  [{idx :index, nm :name, ver :version}]
  (let [target [:field-get "x" (or nm idx)]]
    (if ver
      (wrap-map target "x" [:method-call "x" "into" []])
      [:method-call target "into" []])))

(defn mutable
  [ty]
  (list "mutable" ty))

(defn struct-decl
  [[nm fields]]
  [:struct
   {:attrs {:derive ["Debug"]}}
   nm
   (cond->> (mapv struct-field fields)
     (named? fields) (append ["validity" ["Option" "Bitmap"]]))])

(defn struct-impl
  [[nm fields]]
  [:impl nm [(transpose-one-fn nm fields)]])

(defn struct-from-impl
  [[nm fields]]
  [:impl
   {:for nm}
   ["From" (mutable nm)]
   [[:fn
     {:ret "Self"}
     "from"
     [["x" (mutable nm)]]
     [:block
      [:struct-init "Self" (cond->> (mapv (juxt :name into-immutable) fields)
                             (named? fields) (append ["validity"
                                                      [:method-call
                                                       [:field-get "x" "validity"]
                                                       "map"
                                                       [[:closure
                                                         [["v"]]
                                                         [[:method-call "v" "into" []]]]]]]))]]]]])

(defn -main []
  (doseq [decl (mapcat (juxt struct-decl struct-impl struct-from-impl) (read-structs))]
    (println (emit-expr decl) "\n")))
