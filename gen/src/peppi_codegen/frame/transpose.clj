(ns peppi-codegen.frame.transpose
  (:require
   [clojure.java.io :as io]
   [peppi-codegen.common :refer :all]))

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

(defn -main [path]
  (let [decls (->> (read-json path)
                   (sort-by key)
                   (mapv struct-decl))]
    (println do-not-edit)
    (println (slurp (io/resource "preamble/frame/transpose.rs")))
    (println)
    (doseq [decl decls]
      (println (emit-expr decl) "\n"))))
