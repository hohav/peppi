(ns peppi-codegen.frame.mutable
  (:require
   [clojure.data.json :as json]
   [clojure.java.io :as io]
   [clojure.string :as str]
   [clojure.pprint :refer [pprint]]
   [peppi-codegen.common :refer :all]
   [peppi-codegen.frame.common :refer :all]
   [peppi-codegen.frame.immutable.mod :as immutable]))

(defn array-type
  [ty]
  (cond
    (primitive-types ty) ["Vec" ty]
    (nil? ty)            (throw (ex-info "MutableNullArray" {}))
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

(defn with-capacity
  [{ty :type, ver :version :as m}]
  (let [expr (cond
               (primitive-types ty) [:fn-call "Vec" "with_capacity" ["capacity"]]
               ty                   (with-capacity-custom ty)
               :else                (throw (ex-info "with-capacity-null" {})))]
    (if ver
      [:method-call
       [:method-call "version" "gte" ver]
       "then"
       [[:closure [] [expr]]]]
      expr)))

(defn with-capacity-fn
  [fields]
  (let [null-buffer-init [:fn-call "NullBufferBuilder" "new" ["capacity"]]]
    [:fn
     {:ret "Self"}
     "with_capacity"
     [["capacity" "usize"]
      ["version" "Version"]]
     [:block
      [:struct-init
       "Self"
       (cond->> (mapv (juxt :name with-capacity) fields)
         (named? fields) (append ["validity" null-buffer-init]))]]]))

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

(defn append-default-null
  [target]
  [:method-call target "append_empty_value"])

(defn append-default
  [{nm :name, ty :type, ver :version, idx :index}]
  (let [target (cond-> [:field-get "self" (or nm idx)]
                 ver ((comp unwrap as-mut)))]
    (cond
      (types ty) (append-default-primitive target ty)
      ty         (append-default-composite target)
      :else      (append-default-null target))))

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
                             "append_non_null"]])
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

(defn read-append-null
  [target]
  [:method-call target "append_empty_value"])

(defn read-append
  [{nm :name, ty :type, ver :version, idx :index}]
  (let [target (cond-> [:field-get "self" (or nm idx)]
                 ver ((comp unwrap as-mut)))]
    (cond
      (primitive-types ty) (read-append-primitive target ty)
      ty                   (read-append-composite target)
      :else                (read-append-null target))))

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
     (named? fields) (append [:method-call [:field-get "self" "validity"] "append_non_null"])
     true (append [:struct-init "Ok" [[nil [:unit]]]]))])

(defn finish
  [{idx :index, nm :name, ver :version, ty :type}]
  (let [target [:field-get "self" (or nm idx)]]
    (if (primitive-types ty)
      [:fn-call "mem" "take" [[:ref {:mut true} target]]]
      (if ver
        (wrap-map (as-mut target) "x" [:method-call "x" "finish"])
        [:method-call target "finish"]))))

(defn finish-fn
  [nm fields]
  [:fn
   {:visibility "pub"
    :ret (list "immutable" nm)}
   "finish"
   [["&mut self"]]
   [:block
    [:struct-init
     (list "immutable" nm)
     (cond->> (mapv (juxt :name finish) fields)
       (named? fields) (append ["validity"
                                [:method-call
                                 [:field-get "self" "validity"]
                                 "finish"]]))]]])

(defn struct-field
  [{nm :name, ty :type, ver :version, desc :description}]
  [:struct-field
   {:docstring (field-docstring desc ver)}
   nm
   (cond->> (array-type ty)
     ver (conj ["Option"]))])

(defn tuple-struct-field
  [{ty :type, ver :version}]
  [:tuple-struct-field
   (cond->> (array-type ty)
     ver (conj ["Option"]))])

(defmulti struct-decl
  (fn [[nm {:keys [fields]}]]
    (named? fields)))

(defmethod struct-decl true
  [[nm {:keys [description fields]}]]
  [:struct
   {:docstring description}
   nm
   (->> (mapv struct-field fields)
        (append [:struct-field
                 {:docstring "Indicates which indexes are valid. Invalid indexes can occur on frames where a character is absent (ICs or 2v2 games)"}
                 "validity"
                 ["NullBufferBuilder"]]))])

(defmethod struct-decl false
  [[nm {:keys [description fields]}]]
  [:tuple-struct
   {:docstring description}
   nm
   (mapv tuple-struct-field fields)])

(defn struct-impl
  [[nm {:keys [fields]}]]
  [:impl nm [(with-capacity-fn fields)
             (len-fn fields)
             (append-default-fn fields)
             (read-append-fn fields)
             (finish-fn nm fields)
             (immutable/transpose-one-fn nm fields "values_slice")]])

(defn -main []
  (doseq [decl (mapcat (juxt struct-decl struct-impl) (read-structs))]
    (println (emit-expr decl) "\n")))
