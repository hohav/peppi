(ns peppi-codegen.frame.immutable.slippi
  (:require
   [clojure.java.io :as io]
   [peppi-codegen.common :refer :all]
   [peppi-codegen.frame.common :refer :all]))

(defn use-statement
  [[nm _]]
  [:use (list "crate" "frame" "immutable" nm)])

(defn write-field-primitive
  [target {ty :type}]
  [:method-call
   {:unwrap true
    :generics (when-not (#{"u8" "i8"} ty) ["BE"])}
   "w"
   (str "write_" ty)
   [[:method-call
     target
     "value"
     ["i"]]]])

(defn write-field-composite
  [target field]
  [:method-call
   {:unwrap true}
   target
   "write"
   ["w" "version" "i"]])

(defn write-field
  [{idx :index, nm :name, ty :type, ver :version, :as field}]
  (let [target (cond-> [:field-get "self" (or nm idx)]
                 ver ((comp unwrap as-ref)))]
    (cond
      (primitive-types ty) (write-field-primitive target field)
      ty                   (write-field-composite target field))))

(defn write-fn
  [fields]
  [:fn
   {:ret ["Result" "()"]
    :generics ["W: Write"]}
   "write"
   [["&self"]
    ["w" "&mut W"]
    ["version" "Version"]
    ["i" "usize"]]
   (->> fields
        (nested-version-ifs write-field)
        (into [:block])
        (append [:struct-init "Ok" [[nil [:unit]]]]))])

(defn size-increment
  [{nm :name, ty :type, idx :index}]
  [:op "+=" "size" (if (primitive-types ty)
                     [:fn-call {:generics [ty]} nil "size_of" []]
                     [:fn-call ty "size" ["version"]])])

(defn size-fn
  [fields]
  [:fn
   {:ret "usize"
    :visibility "pub(crate)"}
   "size"
   [["version" "Version"]]
   (->> fields
        (nested-version-ifs size-increment)
        (into [:block [:let {:mutable true} "size" "0usize"]])
        (append "size"))])

(defn struct-impl
  [[nm {:keys [fields]}]]
  [:impl nm [(write-fn fields)
             (size-fn fields)]])

(defn -main []
  (doseq [decl (mapcat (juxt use-statement struct-impl) (read-structs))]
    (println (emit-expr decl) "\n")))
