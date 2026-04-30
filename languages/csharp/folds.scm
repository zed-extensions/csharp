(class_declaration
  body: (_ "{" "}")) @fold

(struct_declaration
  body: (_ "{" "}")) @fold

(interface_declaration
  body: (_ "{" "}")) @fold

(enum_declaration
  body: (_ "{" "}")) @fold

(record_declaration
  body: (_ "{" "}")) @fold

(namespace_declaration
  body: (_ "{" "}")) @fold

(method_declaration
  body: (_ "{" "}")) @fold

(property_declaration
  body: (_ "{" "}")) @fold

(accessor_list
  "{" "}") @fold

(initializer_expression
  "{" "}") @fold

(lambda_expression
  body: (_ "{" "}")) @fold

(switch_statement
  "{" "}") @fold

(preprocessor_if
  ("#if") @fold
  ("#endif") @fold) @fold

(preprocessor_region
  ("#region") @fold
  ("#endregion") @fold) @fold

(comment) @fold
