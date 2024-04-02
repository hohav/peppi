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
  (fn [[nm fields]]
    (named? fields)))

(defmethod struct-decl true
  [[nm fields]]
  [:struct
   {:attrs {:derive ["PartialEq" "Clone" "Copy" "Debug", "Default"]}}
   nm
   (->> fields
        (filter :type)
        (mapv struct-field))])

(defmethod struct-decl false
  [[nm fields]]
  [:tuple-struct
   {:attrs {:derive ["PartialEq" "Clone" "Copy" "Debug", "Default"]}}
   nm
   (->> fields
        (filter :type)
        (mapv tuple-struct-field))])

(defn -main []
  (doseq [decl (mapv struct-decl (read-structs))]
    (println (emit-expr decl) "\n")))
