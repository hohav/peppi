(ns peppi-codegen.frame.mutable
  (:require
   [clojure.data.json :as json]
   [clojure.java.io :as io]
   [clojure.string :as str]
   [clojure.pprint :refer [pprint]]
   [peppi-codegen.common :refer :all]
   [peppi-codegen.frame.immutable :as immutable]))

(defn array-type
  [ty]
  (cond
    (primitive-types ty) ["MutablePrimitiveArray" ty]
    (nil? ty)            "MutableNullArray"
    :else                ty))

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

(defn with-capacity-null
  []
  [:fn-call
   "MutableNullArray"
   "new"
   ["DataType::Null" 0]])

(defn with-capacity
  [{ty :type, ver :version :as m}]
  (let [expr (cond
               (primitive-types ty) (-> ty array-type with-capacity-arrow)
               ty                   (with-capacity-custom ty)
               :else                (with-capacity-null))]
    (if ver
      [:method-call
       [:method-call "version" "gte" ver]
       "then"
       [[:closure [] [expr]]]]
      expr)))

(defn with-capacity-fn
  [fields]
  (let [bitmap-init [:fn-call "MutableBitmap" "with_capacity" ["capacity"]]]
    [:fn
     {:ret "Self"}
     "with_capacity"
     [["capacity" "usize"]
      ["version" "Version"]]
     [:block
      [:struct-init
       "Self"
       (cond->> (mapv (juxt :name with-capacity) fields)
         (named? fields) (append ["validity"
                                   (if (every? :version fields)
                                     [:method-call
                                      [:method-call "version" "lt" (:version (first fields))]
                                      "then"
                                      [[:closure
                                        []
                                        [[:fn-call "MutableBitmap" "with_capacity" ["capacity"]]]]]]
                                     "None")]))]]]))

(defn push-null-primitive
  [target]
  [:method-call target "push_null"])

(defn push-null-composite
  [target]
  [:method-call target "push_null" ["version"]])

(defn push-null-null
  [target]
  [:method-call target "push_null"])

(defn push-null
  [{nm :name, ty :type, ver :version, idx :index}]
  (let [target (cond-> [:field-get "self" (or nm idx)]
                 ver ((comp unwrap as-mut)))]
    (cond
      (types ty) (push-null-primitive target)
      ty         (push-null-composite target)
      :else      (push-null-null target))))

(defn push-null-fn
  [fields]
  [:fn
   {:visibility "pub"}
   "push_null"
   [["&mut self"]
    ["version" "Version"]]
   (cond-> [:block]
     (named? fields) (conj [:let "len" [:method-call "self" "len"]])
     (named? fields) (conj [:method-call
                            [:method-call
                             [:field-get "self" "validity"]
                             "get_or_insert_with"
                             [[:closure
                               []
                               [[:fn-call "MutableBitmap" "from_len_set" ["len"]]]]]]
                            "push"
                            ["false"]])
     true (into (nested-version-ifs push-null fields)))])


(defn read-push-primitive
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
       [[:struct-init "Some" [[nil "x"]]]]]]]]])

(defn read-push-composite
  [target]
  [:method-call
   {:unwrap true}
   target
   "read_push"
   ["r" "version"]])

(defn read-push-null
  [target]
  [:method-call target "push_null"])

(defn read-push
  [{nm :name, ty :type, ver :version, idx :index}]
  (let [target (cond-> [:field-get "self" (or nm idx)]
                 ver ((comp unwrap as-mut)))]
    (cond
      (primitive-types ty) (read-push-primitive target ty)
      ty                   (read-push-composite target)
      :else                (read-push-null target))))

(defn len-fn
  [[{nm :name, idx :index} :as fields]]
  [:fn
   {:visibility "pub"
    :ret "usize"}
   "len"
   [["&self"]]
   [:block
    (if (every? :version fields)
      [:method-call
       [:method-call
        [:method-call
         [:field-get "self" "validity"]
         "as_ref"]
        "map"
        [[:closure [["v"]] [[:method-call "v" "len"]]]]]
       "unwrap_or_else"
       [[:closure
         []
         [[:method-call
           [:method-call
            [:method-call
             [:field-get "self" (or nm idx)]
             "as_ref"]
            "unwrap"]
           "len"]]]]]
      [:method-call [:field-get "self" (or nm idx)] "len"])]])

(defn read-push-fn
  [fields]
  [:fn
   {:visibility "pub"
    :ret ["Result" "()"]}
   "read_push"
   [["&mut self"]
    ["r" "&mut &[u8]"]
    ["version" "Version"]]
   (cond->> (into [:block] (nested-version-ifs read-push fields))
     (named? fields) (append [:method-call
                              [:method-call
                               [:field-get "self" "validity"]
                               "as_mut"]
                              "map"
                              [[:closure
                                [["v"]]
                                [[:method-call "v" "push" ["true"]]]]]])
     true (append [:struct-init "Ok" [[nil [:unit]]]]))])

(defn struct-field
  [{nm :name, ty :type, ver :version}]
  [nm (cond->> (array-type ty)
        ver (conj ["Option"]))])

(defn struct-decl
  [[nm fields]]
  [:struct nm (cond->> (mapv struct-field fields)
                (named? fields) (append ["validity" ["Option" "MutableBitmap"]]))])

(defn struct-impl
  [[nm fields]]
  [:impl nm [(with-capacity-fn fields)
             (len-fn fields)
             (push-null-fn fields)
             (read-push-fn fields)
             (immutable/transpose-one-fn nm fields)]])

(defn normalize-field
  [idx field]
  (-> field
      (update :version #(some-> %
                                (str/split #"\.")
                                vec))
      (assoc :index idx)))

(defn -main [path]
  (let [json (-> (read-json path)
                 (update-vals #(map-indexed normalize-field %))
                 (->> (sort-by key)))
        decls (mapcat (juxt struct-decl struct-impl) json)]
    (println do-not-edit)
    (println (slurp (io/resource "preamble/frame/mutable.rs")))
    (println)
    (doseq [decl decls]
      (println (emit-expr decl) "\n"))))
