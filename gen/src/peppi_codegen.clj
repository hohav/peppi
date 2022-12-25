(ns peppi-codegen
  (:require
   [clojure.data.json :as json]
   [clojure.java.io :as io]
   [clojure.pprint :refer [pprint pp]]
   [clojure.string :as str]))

(defmacro assert!
  ([x]
   (if *assert*
     `(or ~x (throw (new AssertionError (str "Assert failed: " (pr-str '~x)))))
     `(do ~x)))
  ([x msg]
   (if *assert*
     `(or ~x (throw (new AssertionError (str "Assert failed: " ~msg "\n" (pr-str '~x)))))
     `(do ~x))))

(defn kv
  [k v]
  (clojure.lang.MapEntry. k v))

(defn append
  [x coll]
  {:pre [(vector? coll)]}
  (conj coll x))

(defn normalize
  [[nm & more]]
  (let [[props children]
        (if (map? (first more))
          [(first more) (rest more)]
          [{} more])]
    (into [nm props] children)))

(defn pget
  [m k]
  (-> m normalize (get-in [1 k])))

(defn passoc
  [m k v]
  (-> m normalize (assoc-in [1 k] v)))

(def types
  {"bool" "DataType::Boolean"
   "i8"   "DataType::Int8"
   "u8"   "DataType::UInt8"
   "i16"  "DataType::Int16"
   "u16"  "DataType::UInt16"
   "i32"  "DataType::Int32"
   "u32"  "DataType::UInt32"
   "i64"  "DataType::Int64"
   "u64"  "DataType::UInt64"
   "f32"  "DataType::Float32"
   "f64"  "DataType::Float64"})

(def reserved-idents
  #{"type"})

(defn mutable
  [ty]
  (str "Mutable" ty))

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
  [{nm :name, ty :type, :keys [optional version]}]
  [nm (cond->> (mutable-array-type ty)
        version (conj ["Option"]))])

(defn immutable-struct-field
  [{nm :name, ty :type, :keys [optional version]}]
  [nm (cond->> (immutable-array-type ty)
        ;(types ty) (conj ["Box"])
        version    (conj ["Option"]))])

(defn wrap-transpose
  [call]
  [:method-call {:unwrap true} call "transpose"])

(defn wrap-map
  [target binding-name method-call]
  {:pre [(#{:method-call :fn-call} (first method-call))]}
  (let [map-call [:method-call
                  target ;[:method-call target "as_mut"]
                  "map"
                  [[:closure
                    [[binding-name]]
                    [(passoc method-call :unwrap false)]]]]]
    (cond-> map-call
      (pget method-call :unwrap) wrap-transpose)))

(defn as-mut
  [x]
  [:method-call x "as_mut"])

(defn as-ref
  [x]
  [:method-call x "as_ref"])

(defn unwrap
  [x]
  [:method-call x "unwrap"])

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

(defn tuple-struct?
  [fields]
  (not (:name (first fields))))

(defn with-capacity-fn
  [fields]
  [:fn
   {:ret "Self"}
   "with_capacity"
   [["capacity" "usize"]
    ["version" "Version"]]
   [:block
    (if (tuple-struct? fields)
      [:tuple-struct-init
       "Self"
       (mapv with-capacity fields)]
      [:struct-init
       "Self"
       (mapv (juxt :name with-capacity) fields)])]])

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
       [[:tuple-struct-init "Some" ["x"]]]]]]]])

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
        (partition-by :version)
        reverse
        (reduce (fn [acc fields]
                  (let [ver (:version (first fields))
                        stmts (concat (mapv read-push fields) acc)]
                    (if ver
                      [(if-ver ver (into [:block] stmts))]
                      stmts)))
                [])
        (into [:block])
        (append [:tuple-struct-init "Ok" [[:unit]]]))])

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
        (append [:tuple-struct-init "Ok" [[:unit]]]))])

(defn arrow-field
  [{nm :name, ty :type, idx :index}]
  [:fn-call
   "Field"
   "new"
   [[:string (or nm (str "_" idx))]
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

; fn from_struct_array(array: StructArray, version: Version) -> Self {
;     let (fields, values, _) = array.into_data();
;     Self {
;         random_seed: Some(values[0].as_any().downcast_ref::<PrimitiveArray<u32>>().unwrap().clone()),
;         scene_frame_counter: Some(values[1].as_any().downcast_ref::<PrimitiveArray<u32>>().unwrap().clone())
;
;     }
; }

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
                 [:index "values" idx])
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
    (if (tuple-struct? fields)
      [:tuple-struct-init
       "Self"
       (mapv from-struct-array fields)]
      [:struct-init
       "Self"
       (mapv (juxt :name from-struct-array) fields)])]])

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
    [:tuple-struct-init
     "DataType::Struct"
     ["fields"]]]])

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
              (write-fn fields)]]
   [:impl
    {:for nm}
    ["From" (mutable nm)]
    [[:fn
      {:ret "Self"}
      "from"
      [["x" (mutable nm)]]
      [:block
       (if (tuple-struct? fields)
         [:tuple-struct-init "Self" (mapv into-immutable fields)]
         [:struct-init "Self" (mapv (juxt :name into-immutable) fields)])]]]]])

(defn mutable-struct
  [[nm fields]]
  [[:struct (mutable nm) (mapv mutable-struct-field fields)]
   [:impl (mutable nm) [(with-capacity-fn fields)
                        (read-push-fn fields)]]])

(defmulti emit-expr*
  (fn [props & _]
    (:type props)))

(defn emit-expr
  [m]
  (cond
    (nil? m) ""
    (string? m) m
    (number? m) m
    :else
    (let [[ty props & children] (normalize m)]
      (apply emit-expr* (assoc props :type ty) children))))

(defn emit-ident
  [ident]
  (cond->> ident
    (reserved-idents ident) (str "r#")))

(defn emit-type
  [x]
  (if (or (string? x) (keyword? x))
    x
    (let [[ty & generics] x]
      (format "%s::<%s>"
              ty
              (str/join ", " (mapv emit-type generics))))))

(defn emit-fn-body
  [statements]
  (->> statements
       (mapv emit-expr)
       (str/join ";\n")))

(defn emit-fn-arg
  [[nm ty]]
  (if ty
    (format "%s: %s" nm (emit-type ty))
    (emit-ident nm)))

(defmethod emit-expr* :raw
  [_ source]
  source)

(defmethod emit-expr* :unit
  [_]
  "()")

(defmethod emit-expr* :string
  [_ s]
  (format "\"%s\"" s)) ; FIXME: escape

(defmethod emit-expr* :block
  [_ & stmts]
  (->> stmts
       (mapv emit-expr)
       (str/join ";\n")
       (format "{ %s }")))

(defmethod emit-expr* :if
  [_ expr then & [else]]
  (str "if "
       (emit-expr expr)
       (emit-expr then)
       (some->> else emit-expr (str "else "))))

(defmethod emit-expr* :op
  [_ op lhs rhs]
  (format "(%s %s %s)" (emit-expr lhs) op (emit-expr rhs)))

(defmethod emit-expr* :index
  [_ target idx]
  (format "%s[%d]" target idx))

(defmethod emit-expr* :field-get
  [_ target field]
  (format "%s.%s"
          (emit-expr target)
          (emit-ident field)))

(defmethod emit-expr* :method-call
  ([props target nm]
   (emit-expr* props target nm []))
  ([{:keys [generics unwrap]} target nm args]
   (format "%s.%s%s(%s)%s"
           (emit-expr target)
           nm
           (or (some->> generics
                        (mapv emit-type)
                        (str/join ", ")
                        (format "::<%s>"))
               "")
           (->> args
                (mapv emit-expr)
                (str/join ","))
           (if unwrap "?" ""))))

(defmethod emit-expr* :fn-call
  [{:keys [unwrap]} target nm args]
  (format "%s%s(%s)"
          (or (some-> target emit-type (str "::")) "")
          nm
          (->> args
               (mapv emit-expr)
               (str/join ", "))
          (if unwrap "?" "")))

(defmethod emit-expr* :vec!
  [_ args]
  (format "vec![%s]"
          (->> args
               (mapv emit-expr)
               (str/join ", "))))

(defn emit-struct-field-init
  [[nm ty]]
  (format "%s: %s" (emit-ident nm) (emit-expr ty)))

(defmethod emit-expr* :struct-init
  [_ nm fields]
  (format "%s { %s }"
          nm
          (->> fields
               (mapv emit-struct-field-init)
               (str/join ", "))))

(defmethod emit-expr* :tuple-struct-init
  [_ nm fields]
  (format "%s(%s)"
          nm
          (->> fields
               (mapv emit-expr)
               (str/join ", "))))

(defmethod emit-expr* :closure
  [_ args body]
  (format "%s { %s }"
          (->> args
               (mapv emit-fn-arg)
               (str/join ", ")
               (format "|%s|"))
          (emit-fn-body body)))

(defmethod emit-expr* :fn
  [{:keys [ret generics visibility]} nm args body]
  {:pre [(= :block (first body))]}
  (format "%s fn %s%s(%s)%s %s\n"
          (or visibility "")
          nm
          (or (some->> generics (str/join ", ") (format "<%s>")) "")
          (->> args
               (mapv emit-fn-arg)
               (str/join ", "))
          (or (some->> ret emit-type (str " -> ")) "")
          (emit-expr body)))

(defmethod emit-expr* :impl
  [props ty fns]
  (->> fns
       (mapv emit-expr)
       (str/join "\n")
       (format "impl %s%s {\n%s\n}"
               (emit-type ty)
               (or (some->> props :for emit-type (str " for "))
                   ""))))

(defn emit-struct-field
  [[nm ty]]
  (format "\tpub %s: %s," (emit-ident nm) (emit-type ty)))

(defmethod emit-expr* :struct
  [_ nm fields]
  (if (ffirst fields)
    (->> fields ; normal struct
         (mapv emit-struct-field)
         (str/join "\n")
         (format "pub struct %s {\n%s\n}" nm))
    (->> fields ; tuple struct
         (mapv (comp emit-type second))
         (str/join ", ")
         (format "pub struct %s (%s);" nm))))

(defmethod emit-expr* :let
  [props nm expr]
  (format "let%s %s = %s;"
          (if (:mutable props)
            " mut"
            "")
          (if (coll? nm)
            (format "(%s)" (str/join ", " nm))
            nm)
          (emit-expr expr)))

(defn normalize-field
  [idx field]
  (-> field
      (update :version #(some-> %
                                (str/split #"\.")
                                vec))
      (assoc :index idx)))

(defn -main [path]
  (let [json (-> path
                 io/reader
                 (json/read :key-fn keyword, :bigdec true)
                 (update-keys name)
                 (update-vals #(map-indexed normalize-field %)))
        decls (concat (mapcat mutable-struct json)
                      (mapcat immutable-struct json))]
    (println "// This file is auto-generated by `gen/scripts/regen`. Do not edit.\n"
             "\n"
             (slurp (io/resource "preamble.rs"))
             "\n")
    (doseq [decl decls]
      (println (emit-expr decl) "\n"))))
