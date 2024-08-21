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
  (if (types ty)
	["PrimitiveBuilder" (arrow-type ty)]
	ty))

(defn with-capacity-arrow
  [array-type]
  [:fn-call
   array-type
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
               (-> ty array-type with-capacity-arrow)
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
     (cond->> (mapv (juxt :name with-capacity) fields)
       (named? fields) (append ["validity"
                                [:fn-call "NullBufferBuilder" "new" ["capacity"]]]))]]])

(defn push-default-primitive
  [target]
  [:method-call target "append_value" [[:fn-call "Default" "default"]]])

(defn push-default-composite
  [target]
  [:method-call target "push_default" ["version"]])

(defn push-default
  [{nm :name, ty :type, ver :version, idx :index}]
  (let [target (cond-> [:field-get "self" (or nm idx)]
                 ver ((comp unwrap as-mut)))]
    (if (types ty)
	  (push-default-primitive target)
      (push-default-composite target))))

(defn push-default-fn
  [fields]
  [:fn
   {:visibility "pub"}
   "push_default"
   [["&mut self"]
    ["version" "Version"]]
   (cond-> [:block]
     (named? fields) (conj [:method-call
                            [:field-get "self" "validity"]
                            "append"
                            ["true"]])
     true (into (nested-version-ifs push-default fields)))])

(defn push-null-primitive
  [target]
  [:method-call target "append_null"])

(defn push-null-composite
  [target]
  [:method-call target "push_null" ["version"]])

(defn push-null
  [{nm :name, ty :type, ver :version, idx :index}]
  (let [target (cond-> [:field-get "self" (or nm idx)]
                 ver ((comp unwrap as-mut)))]
    (if (types ty)
	  (push-null-primitive target)
      (push-null-composite target))))

(defn push-null-fn
  [fields]
  [:fn
   {:visibility "pub"}
   "push_null"
   [["&mut self"]
    ["version" "Version"]]
   (cond-> [:block]
     (named? fields) (conj [:method-call
                            [:field-get "self" "validity"]
                            "append"
                            ["false"]])
     true (into (nested-version-ifs push-default fields)))])

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
     [[:method-call target "append_value" ["x"]]]]]])

(defn read-push-composite
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
	  (read-push-primitive target ty)
      (read-push-composite target))))

(defn len-fn
  [fields]
  [:fn
   {:visibility "pub"
    :ret "usize"}
   "len"
   [["&self"]]
   [:block
	(if (named? fields)
      [:method-call
       [:field-get "self" "validity"]
       "len"]
	  [:method-call
	   [:field-get "self" "0"]
	   "len"])]])

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
                              [:field-get "self" "validity"]
                              "append"
                              ["true"]])
     true (append [:struct-init "Ok" [[nil [:unit]]]]))])

(defn immutable
  [ty]
  (list "immutable" ty))

(defn finish
  [{idx :index, nm :name, ver :version}]
  (let [target [:field-get "self" (or nm idx)]]
    (if ver
      (wrap-map (as-mut target) "x" [:method-call "x" "finish" []])
      [:method-call target "finish" []])))

(defn finish-fn
  [ty fields]
  [:fn
   {:ret (immutable ty)}
   "finish"
   [["&mut self"]]
   [:block
    [:struct-init
     (immutable ty)
     (cond->> (mapv (juxt :name finish) fields)
       (named? fields)
       (append ["validity"
                [:method-call
                 [:field-get "self" "validity"]
                 "finish"
                 []]]))]]])

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
                 {:docstring "Indicates which indexes are valid (`None` means \"all valid\"). Invalid indexes can occur on frames where a character is absent (ICs or 2v2 games)"}
                 "validity"
                 "NullBufferBuilder"]))])

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
             (push-default-fn fields)
             (read-push-fn fields)
             (finish-fn nm fields)
             (immutable/transpose-one-fn nm fields)]])

(defn -main []
  (doseq [decl (mapcat (juxt struct-decl struct-impl) (read-structs))]
    (println (emit-expr decl) "\n")))
