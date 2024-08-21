(ns peppi-codegen.frame.common
  (:require
   [peppi-codegen.common :refer [read-json]]
   [clojure.string :as str]))

(defn arrow-type
  [ty]
  (case ty
    "i8" "Int8Type"
    "u8" "UInt8Type"
    "i16" "Int16Type"
    "u16" "UInt16Type"
    "i32" "Int32Type"
    "u32" "UInt32Type"
    "f32" "Float32Type"))

(defn field-docstring
  [desc ver]
  (some-> desc
    (cond->> ver (format "*Added: v%s.%s* %s" (ver 0) (ver 1)))))

(defn normalize-field
  [idx field]
  (-> field
      (update :version #(some-> %
                                (str/split #"\.")
                                vec))
      (assoc :index idx)))

(defn read-structs
  []
  (-> (read-json "frames.json")
      (update-vals (fn [s]
                     (update s :fields #(map-indexed normalize-field %))))
      (->> (sort-by key))))
