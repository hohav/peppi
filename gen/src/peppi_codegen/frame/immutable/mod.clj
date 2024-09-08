(ns peppi-codegen.frame.immutable.mod
  (:require
   [clojure.java.io :as io]
   [peppi-codegen.common :refer :all]
   [peppi-codegen.frame.common :refer :all]))

(defn array-type
  [ty]
  (cond
    (primitive-types ty) ["PrimitiveArray" (arrow-type ty)]
    (nil? ty)            "NullArray"
    :else                ty))

(defn struct-field
  [{nm :name, ty :type, ver :version, desc :description}]
  [:struct-field
   {:docstring (field-docstring desc ver)}
   nm
   (cond->> (array-type ty)
     ver (conj ["Option"]))])

(defn tuple-struct-field
  [{ty :type, ver :version, desc :description}]
  [:tuple-struct-field
   {:docstring desc}
   (cond->> (array-type ty)
     ver (conj ["Option"]))])

(defn transpose-one-field-init
  [{idx :index, nm :name, ty :type, ver :version}]
  (let [real-target [:field-get "self" (or nm idx)]
        target (if ver "x" real-target)
        value (if (primitive-types ty)
                [:method-call target "value" ["i"]]
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

(defmulti struct-decl
  (fn [[nm {:keys [fields]}]]
    (named? fields)))

(defmethod struct-decl true
  [[nm {:keys [description fields]}]]
  [:struct
   {:attrs {:derive ["Debug"]}
    :docstring description}
   nm
   (->> (mapv struct-field fields)
        (append [:struct-field
                 {:docstring "Indicates which indexes are valid (`None` means \"all valid\"). Invalid indexes can occur on frames where a character is absent (ICs or 2v2 games)"}
                 "validity"
                 ["Option" "NullBuffer"]]))])

(defmethod struct-decl false
  [[nm {:keys [description fields]}]]
  [:tuple-struct
   {:attrs {:derive ["Debug"]}
    :docstring description}
   nm
   (mapv tuple-struct-field fields)])

(defn struct-impl
  [[nm {:keys [fields]}]]
  [:impl nm [(transpose-one-fn nm fields)]])

#_(defn mutable
  [ty]
  (list "mutable" ty))

#_(defn into-immutable
  [{idx :index, nm :name, ver :version}]
  (let [target [:field-get "x" (or nm idx)]]
    (if ver
      (wrap-map target "x" [:method-call "x" "finish" []])
      [:method-call target "finish" []])))

#_(defn struct-from-impl
  [[nm {:keys [fields]}]]
  [:impl
   {:for nm}
   ["From" (mutable nm)]
   [[:fn
     {:ret "Self"}
     "from"
     [["x" (mutable nm)]]
     [:block
      [:struct-init
       "Self"
       (cond->> (mapv (juxt :name into-immutable) fields)
         (named? fields)
         (append ["validity"
                  [:method-call
                   [:field-get "x" "validity"]
                   "finish"
                   []]]))]]]]])

(defn -main []
  (doseq [decl (mapcat (juxt struct-decl struct-impl) (read-structs))]
    (println (emit-expr decl) "\n")))
