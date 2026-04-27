(ns peppi-codegen.frame.immutable.mod
  (:require
   [clojure.java.io :as io]
   [peppi-codegen.common :refer :all]
   [peppi-codegen.frame.common :refer :all]))

(defn array-type
  [ty]
  (if (primitive-types ty)
    ["Vec" ty]
    (or ty "NullArray")))

(defn with-capacity
  [{ty :type, ver :version :as m}]
  (let [expr (cond
               (primitive-types ty) [:fn-call "Vec" "with_capacity" ["capacity"]]
               ty                   [:fn-call ty "with_capacity" ["capacity" "version"]]
               :else                (throw (ex-info "unsupported type" {:type ty})))]
    (if ver
      [:method-call
       [:method-call "version" "gte" ver]
       "then"
       [[:closure [] [expr]]]]
      expr)))

(defn with-capacity-fn
  [fields]
  [:fn
   {:ret "Self"}
   "with_capacity"
   [["capacity" "usize"]
    ["version" "Version"]]
   [:block
    [:struct-init
     "Self"
     (cond->> (mapv (juxt :name with-capacity) fields)
       (named? fields) (append ["validity" [:fn-call "Validity" "with_capacity" ["capacity"]]]))]]])

(defn len-fn
  [[{nm :name, idx :index} :as fields]]
  [:fn
   {:visibility "pub"
    :ret "usize"}
   "len"
   [["&self"]]
   [:block
    (if (every? :version fields)
      [:method-call [:field-get "self" "validity"] "len"]
      [:method-call [:field-get "self" (or nm idx)] "len"])]])

(defn append-default-primitive
  [target ty]
  [:method-call target "push" [(zero ty)]])

(defn append-default-composite
  [target]
  [:method-call target "append_default" ["version"]])

(defn append-default
  [{nm :name, ty :type, ver :version, idx :index}]
  (let [target (cond-> [:field-get "self" (or nm idx)]
                 ver ((comp unwrap as-mut)))]
    (cond
      (types ty) (append-default-primitive target ty)
      ty         (append-default-composite target)
      :else      (throw (ex-info "unsupported type" {:type ty})))))

(defn append-default-fn
  [fields]
  [:fn
   {:visibility "pub"}
   "append_default"
   [["&mut self"]
    ["version" "Version"]]
   (cond-> [:block]
     (named? fields) (into [[:method-call
                             [:field-get "self" "validity"]
                             "push"
                             ["true"]]])
     true (into (nested-version-ifs append-default fields)))])

(defn read-append-primitive
  [target ty]
  [:method-call
   {:unwrap true}
   [:method-call
    {:generics (when-not (#{"u8" "i8"} ty) ["BE"])}
    "r"
    (str "read_" ty)]
   "map"
   [[:closure
     [["x"]]
     [[:method-call
       target
       "push"
       ["x"]]]]]])

(defn read-append-composite
  [target]
  [:method-call
   {:unwrap true}
   target
   "read_append"
   ["r" "version"]])

(defn read-append
  [{nm :name, ty :type, ver :version, idx :index}]
  (let [target (cond-> [:field-get "self" (or nm idx)]
                 ver ((comp unwrap as-mut)))]
    (cond
      (primitive-types ty) (read-append-primitive target ty)
      ty                   (read-append-composite target)
      :else                (throw (ex-info "unsupported type" {:type ty})))))

(defn read-append-fn
  [fields]
  [:fn
   {:visibility "pub"
    :ret ["Result" "()"]}
   "read_append"
   [["&mut self"]
    ["r" "&mut &[u8]"]
    ["version" "Version"]]
   (cond->> (into [:block] (nested-version-ifs read-append fields))
     (named? fields) (append [:method-call [:field-get "self" "validity"] "push" ["true"]])
     true (append [:struct-init "Ok" [[nil [:unit]]]]))])

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
  [{idx :index, nm :name, ty :type, ver :version} values-fn-name]
  (let [real-target [:field-get "self" (or nm idx)]
        target (if ver "x" real-target)
        value (if (primitive-types ty)
                [:subscript target "i"]
                [:method-call target "transpose_one" ["i" "version"]])]
    (if ver
      (wrap-map (as-ref real-target) "x" value)
      value)))

(defn transpose-one-fn
  [nm fields values-fn-name]
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
                               (mapv #(vector (:name %) (transpose-one-field-init % values-fn-name))))]]]))

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
                 {:docstring "Indicates which indexes are valid.\nInvalid indexes can occur on frames where a character is absent (ICs or 2v2 games)"}
                 "validity"
                 "Validity"]))])

(defmethod struct-decl false
  [[nm {:keys [description fields]}]]
  [:tuple-struct
   {:attrs {:derive ["Debug"]}
    :docstring description}
   nm
   (mapv tuple-struct-field fields)])

(defn struct-impl
  [[nm {:keys [fields]}]]
  [:impl nm [(with-capacity-fn fields)
             (len-fn fields)
             (append-default-fn fields)
             (read-append-fn fields)
             (transpose-one-fn nm fields "values")]])

(defn -main []
  (doseq [decl (mapcat (juxt struct-decl struct-impl) (read-structs))]
    (println (emit-expr decl) "\n")))
