; Identifiers whose value is worth showing while a debug session is paused.
;
; The `#not-match?` guards drop PascalCase identifiers in positions where a
; type name is as likely as a variable, since evaluating a type produces noise
; rather than a value.

(parameter
  name: (identifier) @debug-variable)

(variable_declarator
  name: (identifier) @debug-variable)

(declaration_expression
  name: (identifier) @debug-variable)

(catch_declaration
  name: (identifier) @debug-variable)

(foreach_statement
  left: (identifier) @debug-variable)

(assignment_expression
  left: (identifier) @debug-variable)

(assignment_expression
  left: (member_access_expression) @debug-variable)

(element_access_expression
  expression: (identifier) @debug-variable)

(member_access_expression
  expression: (identifier) @debug-variable
  (#not-match? @debug-variable "^[A-Z]"))

(argument
  (identifier) @debug-variable
  (#not-match? @debug-variable "^[A-Z]"))

(binary_expression
  (identifier) @debug-variable
  (#not-match? @debug-variable "^[A-Z]"))

(prefix_unary_expression
  (identifier) @debug-variable)

(postfix_unary_expression
  (identifier) @debug-variable)

(return_statement
  (identifier) @debug-variable)

(interpolation
  (identifier) @debug-variable)

(if_statement
  condition: (identifier) @debug-variable)

(while_statement
  condition: (identifier) @debug-variable)

(switch_statement
  value: (identifier) @debug-variable)

(conditional_expression
  condition: (identifier) @debug-variable)

(await_expression
  (identifier) @debug-variable)

[
  (block)
  (declaration_list)
] @debug-scope
