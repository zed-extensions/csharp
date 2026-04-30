(method_declaration
  body: (block) @local-scope)

(local_function_statement
  body: (block) @local-scope)

(property_declaration
  body: (_) @local-scope)

(accessor_declaration
  body: (block) @local-scope)

(for_statement
  body: (_) @local-scope)

(for_each_statement
  body: (_) @local-scope)

(while_statement
  body: (_) @local-scope)

(do_statement
  body: (_) @local-scope)

(if_statement
  body: (_) @local-scope)

(switch_statement
  body: (_) @local-scope)

(using_statement
  body: (_) @local-scope)

(lock_statement
  body: (_) @local-scope)

(lambda_expression
  body: (_) @local-scope)

(anonymous_method_expression
  body: (_) @local-scope)

(variable_declaration
  (variable_declarator
    name: (identifier) @definition))

(parameter
  name: (identifier) @definition)

(parameter
  name: (identifier) @definition.parameter)

(for_each_statement
  (identifier) @definition)

(from_clause
  (identifier) @definition)

(let_clause
  (identifier) @definition)

(identifier) @reference
