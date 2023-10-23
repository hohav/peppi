(ns peppi-codegen.columnar
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
   {:derives ["PartialEq", "Debug"]}
   nm
   (mapv field fields)])

(defn -main [path]
  (let [json (read-json path)
        decls (mapv struct-decl json)]
    (println do-not-edit)
    (println (slurp (io/resource "preamble/columnar.rs")))
    (println)
    (doseq [decl decls]
      (println (emit-expr decl) "\n"))))
