(ns peppi-codegen.frame
  (:require
   [clojure.data.json :as json]
   [clojure.java.io :as io]
   [clojure.string :as str]
   [clojure.pprint :refer [pprint]]
   [peppi-codegen.common :refer :all]))

(defn mutable-array-type
  [ty]
  (if (types ty)
    ["MutablePrimitiveArray" ty]
    (str "Mutable" ty)))

(defn immutable-array-type
  [ty]
  (if (types ty)
    ["PrimitiveArray" ty]
    ty))

(defn mutable-struct-field
  [{nm :name, ty :type, ver :version}]
  [nm (cond->> (mutable-array-type ty)
        ver (conj ["Option"]))])

(defn immutable-struct-field
  [{nm :name, ty :type, ver :version}]
  [nm (cond->> (immutable-array-type ty)
        ;(types ty) (conj ["Box"])
        ver (conj ["Option"]))])

(defn if-ver
  ([ver then]
   (if-ver ver then nil))
  ([ver then else]
   [:if
    [:method-call "version" "gte" ver]
    (cond->> then
      (not= :block (first then)) (conj [:block]))
    (cond->> else
      (and else (not= :block (first else))) (conj [:block]))]))

(defn push-none
  [{nm :name, ty :type, ver :version, idx :index}]
  (let [target (cond-> [:field-get "self" (or nm idx)]
                 ver ((comp unwrap as-mut)))]
    (into [:method-call target]
          (if (types ty)
            ["push" ["None"]]
            ["push_none" ["version"]]))))

(defn nested-version-ifs
  [f fields]
  (->> fields
       (partition-by :version)
       reverse
       (reduce (fn [acc fields]
                 (let [ver (:version (first fields))
                       stmts (concat (mapv f fields) acc)]
                   (if ver
                     [(if-ver ver (into [:block] stmts))]
                     stmts)))
               [])))

(defn with-capacity-arrow
  [arrow-type]
  [:fn-call
   arrow-type
   "with_capacity"
   ["capacity"]])

(defn with-capacity-custom
  [ty]
  [:fn-call
   ty
   "with_capacity"
   ["capacity" "version"]])

(defn with-capacity
  [{ty :type, ver :version :as m}]
  (let [expr (if (types ty)
               (-> ty mutable-array-type with-capacity-arrow)
               (with-capacity-custom (mutable ty)))]
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
     (mapv (juxt :name with-capacity) fields)]]])

(defn primitive-push-none
  [target]
  [:method-call target "push" ["None"]])

(defn composite-push-none
  [target]
  [:method-call target "push_none" ["version"]])

(defn push-none
  [{nm :name, ty :type, ver :version, idx :index}]
  (let [target (cond-> [:field-get "self" (or nm idx)]
                 ver ((comp unwrap as-mut)))]
    (if (types ty)
      (primitive-push-none target)
      (composite-push-none target))))

(defn push-none-fn
  [fields]
  [:fn
   {:visibility "pub"}
   "push_none"
   [["&mut self"]
    ["version" "Version"]]
    (into [:block]
          (nested-version-ifs push-none fields))])

(defn primitive-read-push
  [target ty]
  [:method-call
   {:unwrap true}
   [:method-call
    {:generics (when-not (#{"u8" "i8" "bool"} ty) ["BE"])}
    "r"
    (str "read_" ty)]
   "map"
   [[:closure
     [["x"]]
     [[:method-call
       target
       "push"
       [[:struct-init "Some" [[nil "x"]]]]]]]]])

(defn composite-read-push
  [target]
  [:method-call
   {:unwrap true}
   target
   "read_push"
   ["r" "version"]])

(defn read-push
  [{nm :name, ty :type, ver :version, idx :index}]
  (let [target (cond-> [:field-get "self" (or nm idx)]
                 ver ((comp unwrap as-mut)))]
    (if (types ty)
      (primitive-read-push target ty)
      (composite-read-push target))))

(defn read-push-fn
  [fields]
  [:fn
   {:visibility "pub"
    :ret ["Result" "()"]}
   "read_push"
   [["&mut self"]
    ["r" "&mut &[u8]"]
    ["version" "Version"]]
   (->> fields
        (nested-version-ifs read-push)
        (into [:block])
        (append [:struct-init "Ok" [[nil [:unit]]]]))])

(defn write-field-primitive
  [target {ty :type}]
  [:method-call
   {:unwrap true
    :generics (when-not (#{"u8" "i8" "bool"} ty) ["BE"])}
   "w"
   (str "write_" ty)
   [[:method-call
     target
     "value"
     ["i"]]]])

(defn write-field-composite
  [target field]
  [:method-call
   {:unwrap true}
   target
   "write"
   ["w" "version" "i"]])

(defn write-field
  [{idx :index, nm :name, ty :type, ver :version, :as field}]
  (let [target (cond-> [:field-get "self" (or nm idx)]
                 ver ((comp unwrap as-ref)))]
    (if (types ty)
      (write-field-primitive target field)
      (write-field-composite target field))))

(defn write-fn
  [fields]
  [:fn
   {:ret ["Result" "()"]
    :generics ["W: Write"]}
   "write"
   [["&self"]
    ["w" "&mut W"]
    ["version" "Version"]
    ["i" "usize"]]
   (->> fields
        (nested-version-ifs write-field)
        (into [:block])
        (append [:struct-init "Ok" [[nil [:unit]]]]))])

(defn arrow-field
  [{nm :name, ty :type, idx :index}]
  [:fn-call
   "Field"
   "new"
   [[:string (or nm (str idx))]
    (types ty [:fn-call ty "data_type" ["version"]])
    "false"]])

(defn arrow-values
  [{nm :name, ty :type, idx :index, ver :version}]
  (let [target (cond-> [:field-get "self" (or nm idx)]
                 ver (#(vector :method-call % "unwrap")))]
    (if (types ty)
      [:method-call
       target
       "boxed"]
      [:method-call
       [:method-call target "into_struct_array" ["version"]]
       "boxed"
       []])))

(defn into-struct-array-fn
  [fields]
  [:fn
   {:ret "StructArray"}
   "into_struct_array"
   [["self"]
    ["version" "Version"]]
   [:block
    [:let
     {:mutable true}
     "values"
     [:vec! []]]
    (->> fields
         (nested-version-ifs
          (fn [field]
            [:method-call
             "values"
             "push"
             [(arrow-values field)]]))
         (into [:block]))
    [:fn-call
     "StructArray"
     "new"
     [[:fn-call
       "Self"
       "data_type"
       ["version"]]
      "values"
      "None"]]]])

(defn transpose-one-field-init
  [{idx :index, nm :name, ty :type, ver :version}]
  (let [real-target [:field-get "self" (or nm idx)]
        target (if ver "x" real-target)
        value (if (types ty)
                [:subscript [:method-call target "values"] "i"]
                [:method-call target "transpose_one" ["i" "version"]])]
    (if ver
      (wrap-map (as-ref real-target) "x" value)
      value)))

(defn transpose-one-fn
  [nm fields]
  (let [ctype (list "columnar" nm)]
    [:fn
     {:visibility "pub"
      :ret ctype}
     "transpose_one"
     [["&self"]
      ["i" "usize"]
      ["version" "Version"]]
     [:block
      [:struct-init ctype (mapv (juxt :name transpose-one-field-init) fields)]]]))

(defn downcast-clone
  [target as]
  [:method-call
   [:method-call
    [:method-call
     {:generics [as]}
     [:method-call
      target
      "as_any"]
     "downcast_ref"]
    "unwrap"]
   "clone"])

(defn from-struct-array
  [{ty :type, ver :version, idx :index, :as field}]
  (let [target (if ver
                 [:method-call "values" "get" [idx]]
                 [:subscript "values" idx])
        body (if (types ty)
               (downcast-clone (if ver "x" target) ["PrimitiveArray" ty])
               [:fn-call
                ty
                "from_struct_array"
                [(downcast-clone (if ver "x" target) "StructArray")
                 "version"]])]
    (cond->> body
      ver (wrap-map target "x"))))

(defn from-struct-array-fn
  [fields]
  [:fn
   {:ret "Self"}
   "from_struct_array"
   [["array" "StructArray"]
    ["version" "Version"]]
   [:block
    [:let
     ["_" "values" "_"]
     [:method-call "array" "into_data"]]
     [:struct-init
      "Self"
      (mapv (juxt :name from-struct-array) fields)]]])

(defn data-type-fn
  [fields]
  [:fn
   {:ret "DataType"}
   "data_type"
   [["version" "Version"]]
   [:block
    [:let
     {:mutable true}
     "fields"
     [:vec! []]]
    (->> fields
         (nested-version-ifs
          (fn [f]
            [:method-call
             "fields"
             "push"
             [(arrow-field f)]]))
         (into [:block]))
    [:struct-init
     "DataType::Struct"
     [[nil "fields"]]]]])

(defn into-immutable
  [{idx :index, nm :name, ver :version}]
  (let [target [:field-get "x" (or nm idx)]]
    (if ver
      (wrap-map target "x" [:method-call "x" "into" []])
      [:method-call target "into" []])))

(defn immutable-struct
  [[nm fields]]
  [[:struct nm (mapv immutable-struct-field fields)]
   [:impl nm [(data-type-fn fields)
              (into-struct-array-fn fields)
              (from-struct-array-fn fields)
              (write-fn fields)
              (transpose-one-fn nm fields)]]
   [:impl
    {:for nm}
    ["From" (mutable nm)]
    [[:fn
      {:ret "Self"}
      "from"
      [["x" (mutable nm)]]
      [:block
       [:struct-init "Self" (mapv (juxt :name into-immutable) fields)]]]]]])

(defn mutable-struct
  [[nm fields]]
  [[:struct (mutable nm) (mapv mutable-struct-field fields)]
   [:impl (mutable nm) [(with-capacity-fn fields)
                        (push-none-fn fields)
                        (read-push-fn fields)]]])

(defn normalize-field
  [idx field]
  (-> field
      (update :version #(some-> %
                                (str/split #"\.")
                                vec))
      (assoc :index idx)))

(defn -main [path]
  (let [json (-> (read-json path)
                 (update-vals #(map-indexed normalize-field %)))
        decls (concat (mapcat mutable-struct json)
                      (mapcat immutable-struct json))]
    (println do-not-edit)
    (println (slurp (io/resource "preamble/frame.rs")))
    (println)
    (doseq [decl decls]
      (println (emit-expr decl) "\n"))))
