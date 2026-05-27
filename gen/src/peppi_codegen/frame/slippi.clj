(ns peppi-codegen.frame.slippi
  (:require
   [clojure.java.io :as io]
   [peppi-codegen.common :refer :all]
   [peppi-codegen.frame.common :refer :all]))

(defn use-statement
  [[nm _]]
  [:use (list "crate" "frame" nm)])

(defn read-append-primitive
  [target ty]
  [:method-call
   target
   "push"
   [[:method-call
	 {:unwrap true
      :generics (when-not (#{"u8" "i8"} ty) ["BE"])}
     "r"
     (str "read_" ty)]]])

(defn read-append-composite
  [target]
  [:method-call
   {:unwrap true}
   target
   "read_append"
   ["r" "version"]])

(defn read-append
  [{nm :name, ty :type, ver :version, idx :index}]
  (let [target (cond-> [:field-get "self" (or nm idx)]
                 ver ((comp unwrap as-mut)))]
    (cond
      (primitive-types ty) (read-append-primitive target ty)
      ty                   (read-append-composite target)
      :else                (throw (ex-info "unsupported type" {:type ty})))))

(defn read-append-fn
  [fields]
  [:fn
   {:visibility "pub"
    :ret ["Result" "()"]}
   "read_append"
   [["&mut self"]
    ["r" "&mut &[u8]"]
    ["version" "Version"]]
   (cond->> (into [:block] (nested-version-ifs read-append fields))
     (named? fields) (append [:method-call [:field-get "self" "validity"] "push" ["true"]])
     true (append [:struct-init "Ok" [[nil [:unit]]]]))])

(defn write-field-primitive
  [target {ty :type}]
  [:method-call
   {:unwrap true
    :generics (when-not (#{"u8" "i8"} ty) ["BE"])}
   "w"
   (str "write_" ty)
   [[:subscript target "i"]]])

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
  [:impl nm [(read-append-fn fields)
             (write-fn fields)
             (size-fn fields)]])

(defn -main []
  (doseq [decl (mapcat (juxt use-statement struct-impl) (read-structs))]
    (println (emit-expr decl) "\n")))
