(identifier) @variable

[
  (predefined_type)
  (type_parameter)
] @type

(_ type: (identifier) @type)

(method_declaration name: (identifier) @function)
(invocation_expression function: (identifier) @function)
(invocation_expression function: (member_access_expression name: (identifier) @function))
(property_declaration name: (identifier) @property)

[
  (string_literal)
  (verbatim_string_literal)
  (raw_string_literal)
  (interpolated_string_expression)
  (character_literal)
] @string

(escape_sequence) @string.escape

[
  (integer_literal)
  (real_literal)
] @number

[
  (boolean_literal)
  (null_literal)
] @constant.builtin

[
  (comment)
  (razor_comment)
  (html_comment)
] @comment

[
  "at_page"
  "at_using"
  "at_model"
  "at_rendermode"
  "at_inject"
  "at_implements"
  "at_layout"
  "at_inherits"
  "at_attribute"
  "at_typeparam"
  "at_namespace"
  "at_preservewhitespace"
  "at_addtaghelper"
  "at_removetaghelper"
  "at_taghelperprefix"
  "at_block"
  "at_if"
  "at_switch"
  "at_for"
  "at_foreach"
  "at_while"
  "at_do"
  "at_try"
  "at_section"
  "at_lock"
  (razor_attribute_name)
  (razor_rendermode)
  (modifier)
  (implicit_type)
  "if"
  "else"
  "switch"
  "case"
  "default"
  "for"
  "foreach"
  "in"
  "while"
  "do"
  "try"
  "catch"
  "finally"
  "return"
  "throw"
  "new"
  "await"
  "this"
  "base"
  "get"
  "set"
] @keyword

[
  "at_explicit"
  "at_implicit"
  "at_colon_transition"
  "at_at_escape"
] @punctuation.special

["(" ")" "[" "]" "{" "}"] @punctuation.bracket
[";" "." ","] @punctuation.delimiter
["=" "=>" "+" "-" "*" "/" "!" "==" "!=" "&&" "||" "++" "--" "??" "?"] @operator
