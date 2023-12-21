(ns peppi-codegen.frame.common
  (:require
   [peppi-codegen.common :refer [read-json]]
   [clojure.string :as str]))

(defn normalize-field
  [idx field]
  (-> field
      (update :version #(some-> %
                                (str/split #"\.")
                                vec))
      (assoc :index idx)))

(defn read-structs
  []
  (-> (read-json "structs.json")
      (update-vals #(map-indexed normalize-field %))
      (->> (sort-by key))))

