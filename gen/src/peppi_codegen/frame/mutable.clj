(ns peppi-codegen.frame.mutable
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
    ty))

(defn mutable-struct-field
  [{nm :name, ty :type, ver :version}]
  [nm (cond->> (mutable-array-type ty)
        ver (conj ["Option"]))])

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
               (with-capacity-custom ty))]
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

(defn mutable-struct
  [[nm fields]]
  [[:struct nm (mapv mutable-struct-field fields)]
   [:impl nm [(with-capacity-fn fields)
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
        decls (mapcat mutable-struct json)]
    (println do-not-edit)
    (println (slurp (io/resource "preamble/frame/mutable.rs")))
    (println)
    (doseq [decl decls]
      (println (emit-expr decl) "\n"))))
