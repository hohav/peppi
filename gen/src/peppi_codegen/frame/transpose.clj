(ns peppi-codegen.frame.transpose
  (:require
   [clojure.java.io :as io]
   [peppi-codegen.common :refer :all]
   [peppi-codegen.frame.common :refer :all]))

(defn field
  [{nm :name, ty :type, ver :version}]
  [nm (cond->> ty
        ver (conj ["Option"]))])

(defn struct-decl
  [[nm fields]]
  [:struct
   {:attrs {:derive ["PartialEq" "Clone" "Copy" "Debug", "Default"]}}
   nm
   (->> fields
        (filter :type)
        (mapv field))])

(defn -main []
  (doseq [decl (mapv struct-decl (read-structs))]
    (println (emit-expr decl) "\n")))
