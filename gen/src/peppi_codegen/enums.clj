(ns peppi-codegen.enums
  (:require
   [camel-snake-kebab.core :as csk]
   [clojure.data.json :as json]
   [clojure.java.io :as io]
   [clojure.pprint :refer [pprint]]
   [peppi-codegen.common :refer :all]))

(defn enum
  [[nm {ty :type, values :known_values}]]
  [:enum
   {:attrs {:derive ["Debug" "PartialEq" "Eq" "Clone" "Copy" "TryFromPrimitive"]
            :repr [ty]}}
   nm
   (-> values
       (update-keys (comp bigdec name))
       (->> (sort-by key)
            (mapv (juxt (comp csk/->PascalCase :ident val) key))))])

(defn -main [enum-name]
  (doseq [decl (mapv enum (read-json (format "enums/%s.json" enum-name)))]
    (println (emit-expr decl) "\n")))
