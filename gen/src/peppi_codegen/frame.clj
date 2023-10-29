(ns peppi-codegen.frame
  (:require
   [clojure.data.json :as json]
   [clojure.java.io :as io]
   [clojure.string :as str]
   [clojure.pprint :refer [pprint]]
   [peppi-codegen.common :refer :all]))

(defn immutable-array-type
  [ty]
  (if (types ty)
    ["PrimitiveArray" ty]
    ty))

(defn immutable-struct-field
  [{nm :name, ty :type, ver :version}]
  [nm (cond->> (immutable-array-type ty)
        ver (conj ["Option"]))])

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
  (let [ctype (list "transpose" nm)]
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
     (list "DataType" "Struct")
     [[nil "fields"]]]]])

(defn into-immutable
  [{idx :index, nm :name, ver :version}]
  (let [target [:field-get "x" (or nm idx)]]
    (if ver
      (wrap-map target "x" [:method-call "x" "into" []])
      [:method-call target "into" []])))

(defn mutable
  [ty]
  (list "mutable" ty))

(defn immutable-struct
  [[nm fields]]
  [[:struct {:derives #{"Debug"}} nm (mapv immutable-struct-field fields)]
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
        decls (mapcat immutable-struct json)]
    (println do-not-edit)
    (println (slurp (io/resource "preamble/frame.rs")))
    (println)
    (doseq [decl decls]
      (println (emit-expr decl) "\n"))))
