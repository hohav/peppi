(ns peppi-codegen.frame.transpose
  (:require
   [clojure.java.io :as io]
   [peppi-codegen.common :refer :all]
   [peppi-codegen.frame.common :refer :all]))

(defn struct-field
  [{nm :name, ty :type, ver :version}]
  [:struct-field
   nm
   (cond->> ty
     ver (conj ["Option"]))])

(defn tuple-struct-field
  [{ty :type, ver :version}]
  [:tuple-struct-field
   (cond->> ty
     ver (conj ["Option"]))])

(defmulti struct-decl
  (fn [[nm {:keys [fields]}]]
    (named? fields)))

(defmethod struct-decl true
  [[nm {:keys [fields]}]]
  [:struct
   {:attrs {:derive ["PartialEq" "Clone" "Copy" "Debug", "Default"]}}
   nm
   (->> fields
        (filter :type)
        (mapv struct-field))])

(defmethod struct-decl false
  [[nm {:keys [fields]}]]
  [:tuple-struct
   {:attrs {:derive ["PartialEq" "Clone" "Copy" "Debug", "Default"]}}
   nm
   (->> fields
        (filter :type)
        (mapv tuple-struct-field))])

(defn read-primitive
  [ty]
  [:method-call
   {:unwrap true
    :generics (when-not (#{"u8" "i8"} ty) ["BE"])}
   "r"
   (str "read_" ty)])

(defn read-composite
  [ty]
  [:fn-call {:unwrap true} ty "read" ["r" "version"]])

(defn read-field
  [{nm :name, ty :type, ver :version, idx :index}]
  (let [read-call (if (primitive-types ty)
                    (read-primitive ty)
                    (read-composite ty))]
    (if ver
      [:if [:method-call "version" "gte" ver]
       [:block [:tuple-struct-init "Some" [read-call]]]
       [:block "None"]]
      read-call)))

(defn read-fn
  [fields]
  [:fn
   {:visibility "pub"
    :ret ["Result" "Self"]}
   "read"
   [["r" "&mut &[u8]"]
    ["version" "Version"]]
   [:block
    [:tuple-struct-init "Ok"
     [[:struct-init "Self" (mapv (juxt :name read-field) fields)]]]]])

(defn struct-impl
  [[nm {:keys [fields]}]]
  [:impl nm [(read-fn fields)]])

(defn -main []
  (doseq [s (read-structs)]
    (println (emit-expr (struct-decl s)) "\n\n"
             (emit-expr (struct-impl s)) "\n")))
