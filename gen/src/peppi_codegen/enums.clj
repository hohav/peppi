(ns peppi-codegen.enums
  (:require
   [clojure.data.json :as json]
   [clojure.java.io :as io]
   [clojure.pprint :refer [pprint]]
   [peppi-codegen.common :refer :all]))

(defn enum
  [[nm {ty :type, values :known_values}]]
  [:enum
   {:repr ty}
   nm
   (-> values
       (update-keys (comp bigdec name))
       (->> (sort-by key)
            (mapv (juxt (comp :ident val) key))))])

(defn -main [path]
  (let [json (read-json path)
        decls (mapv enum json)]
    (println do-not-edit)
    (doseq [decl decls]
      (println (emit-expr decl) "\n"))))
