(ns peppi-codegen.frame.immutable.peppi
  (:require
   [clojure.java.io :as io]
   [peppi-codegen.common :refer :all]
   [peppi-codegen.frame.common :refer :all]))

(defn use-statement
  [[nm _]]
  [:use (list "crate" "frame" "immutable" nm)])

(defn arrow-field
  [{nm :name, ty :type, idx :index}]
  [:fn-call
   "Field"
   "new"
   [[:string (or nm (str idx))]
    (types ty [:fn-call ty "data_type" ["version"]])
    "false"]])

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

(defn push-call
  [field]
  [:method-call
   "values"
   "push"
   [(arrow-values field)]])

(defn into-struct-array-fn
  [fields]
  (let [let-values [:let {:mutable true} "values" [:vec! []]]
        struct-init [:fn-call
                     "StructArray"
                     "new"
                     [[:fn-call "Self" "data_type" ["version"]]
                      "values"
                      (if (named? fields) "self.validity" "None")]]]
    [:fn
     {:ret "StructArray"}
     "into_struct_array"
     [["self"]
      ["version" "Version"]]
     (->> (nested-version-ifs push-call fields)
          (into [:block let-values])
          (append struct-init))]))

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
        ver-target (if ver "x" target)
        body (cond
               (primitive-types ty)
               (downcast-clone ver-target ["PrimitiveArray" ty])

               (nil? ty)
               (downcast-clone ver-target "NullArray")

               :else
               [:fn-call
                ty
                "from_struct_array"
                [(downcast-clone ver-target "StructArray")
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
     ["_" "values" "validity"]
     [:method-call "array" "into_data"]]
     [:struct-init
      "Self"
      (cond->> (mapv (juxt :name from-struct-array) fields)
        (named? fields) (append ["validity" "validity"]))]]])

(defn struct-impl
  [[nm {:keys [fields]}]]
  [:impl
   {:for nm}
   "StructArrayConvertible"
   [(data-type-fn fields)
    (into-struct-array-fn fields)
    (from-struct-array-fn fields)]])

(defn -main []
  (doseq [decl (mapcat (juxt use-statement struct-impl) (read-structs))]
    (println (emit-expr decl) "\n")))
