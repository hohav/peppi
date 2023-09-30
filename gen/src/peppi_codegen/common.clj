(ns peppi-codegen.common
  (:require
   [clojure.pprint :refer [pprint]]
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

(defn wrap-transpose
  [call]
  [:method-call {:unwrap true} call "transpose"])

(defn wrap-map
  [target binding-name method-call]
  {:pre [(#{:method-call :fn-call} (first method-call))]}
  (let [map-call [:method-call
                  target
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
