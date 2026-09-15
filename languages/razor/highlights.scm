; Razor-specific highlighting.
;
; The razor grammar is a superset of tree-sitter-c-sharp: all C# nodes keep
; their original names, so the C# query set from zed-extensions/csharp is
; inlined below verbatim (Zed does not support Neovim's `; inherits:`
; directive, which is why upstream queries fail in Zed).

; Comments
[
  (razor_comment)
  (html_comment)
  (comment)
] @comment

; Directives (@page, @using, @inject, ...)
;
; The grammar's directive rules do not accept an optional trailing `;`
; (e.g. `@using System;`), leaving it as an ERROR node; paint those stray
; semicolons so they do not render unhighlighted.
((ERROR) @punctuation.delimiter
  (#eq? @punctuation.delimiter ";"))

; Directives (@page, @using, @inject, ...)
; VS Code scopes these keyword.control.razor.directive.* — same family as
; the other @-transitions, so they all change together if the theme defines
; `keyword.control` (VS Code Dark+ renders them #C586C0 purple).
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
] @keyword.control

; Control-flow transitions — VS Code: keyword.control.razor.* (purple in
; VS Code themes; falls back to `keyword` blue in themes without a
; keyword.control entry).
[
  "at_block"
  "at_lock"
  "at_section"
  "at_if"
  "at_switch"
  "at_for"
  "at_foreach"
  "at_while"
  "at_do"
  "at_try"
  "catch"
  "finally"
  "at_at_escape"
  "at_colon_transition"
] @keyword.control

; Code transitions (@ expressions, @( ... ) blocks) — VS Code:
; keyword.control.cshtml.transition.
[
  "at_implicit"
  "at_explicit"
] @keyword.control

(razor_implicit_expression
  "at_implicit" @keyword.control
  (await_expression
    "await" @keyword.control))

; Blazor component attributes
(razor_attribute_name) @attribute

; HTML attribute names — the razor layer parses these itself, so they stay
; colored even where the injected html layer's parse scrambles (nested
; quotes in attribute values, e.g. attr="@x.ToString("o", ...)").
(html_attribute_name) @attribute

(razor_attribute_value) @string

(razor_rendermode) @property

(taghelper_wildcard) @constant

; ------------------------------------------------------------------
; C# (ported from zed-extensions/csharp, unchanged node names)
; ------------------------------------------------------------------

(identifier) @variable

; Methods
(method_declaration
  name: (identifier) @function)

(_
  function: (identifier) @function)

(local_function_statement
  name: (identifier) @function)

; Types
(interface_declaration
  name: (identifier) @type)

(class_declaration
  name: (identifier) @type)

(enum_declaration
  name: (identifier) @type)

(struct_declaration
  (identifier) @type)

(record_declaration
  (identifier) @type)

; TODO: Let this be @module again once we support fallbacks
(namespace_declaration
  name: (identifier) @type)

(generic_name
  (identifier) @type)

(type_parameter
  (identifier) @property.definition)

(parameter
  type: (identifier) @type)

(type_argument_list
  (identifier) @type)

(as_expression
  right: (identifier) @type)

(is_expression
  right: (identifier) @type)

(constructor_declaration
  name: (identifier) @constructor)

(destructor_declaration
  name: (identifier) @constructor)

(_
  type: (identifier) @type)

(base_list
  (identifier) @type)

(predefined_type) @type.builtin

; Enum
(enum_member_declaration
  (identifier) @property.definition)

; Literals
[
  (real_literal)
  (integer_literal)
] @number

[
  (character_literal)
  (string_literal)
  (raw_string_literal)
  (verbatim_string_literal)
  (interpolated_string_expression)
  (interpolation_start)
  (interpolation_quote)
] @string

(escape_sequence) @string.escape

[
  (boolean_literal)
  (null_literal)
] @constant.builtin

; Tokens
[
  ";"
  "."
  ","
] @punctuation.delimiter

; Tokens. Comparison/assignment operators are scoped to their expression
; nodes below: a bare "<" ">" "=" token list would paint every HTML tag's
; brackets and attribute equals in markup as C# operators.
[
  "--"
  "-"
  "-="
  "&"
  "&="
  "&&"
  "+"
  "++"
  "+="
  "!"
  "=>"
  "?"
  "??"
  "??="
  "|"
  "|="
  "||"
  "^"
  "^="
  "~"
  "*"
  "*="
  "/"
  "/="
  "%"
  "%="
  ":"
] @operator

(binary_expression
  operator: (_) @operator)

(assignment_expression
  operator: (_) @operator)

(unary_expression
  operator: (_) @operator)

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
  (interpolation_brace)
] @punctuation.bracket

; Keywords
[
  (modifier)
  "this"
  (implicit_type)
] @keyword

[
  "add"
  "alias"
  "as"
  "base"
  "break"
  "case"
  "checked"
  "class"
  "continue"
  "default"
  "delegate"
  "do"
  "else"
  "enum"
  "event"
  "explicit"
  "extern"
  "finally"
  "for"
  "foreach"
  "global"
  "goto"
  "if"
  "implicit"
  "interface"
  "is"
  "lock"
  "namespace"
  "notnull"
  "operator"
  "params"
  "return"
  "remove"
  "sizeof"
  "stackalloc"
  "static"
  "struct"
  "switch"
  "throw"
  "try"
  "typeof"
  "unchecked"
  "using"
  "while"
  "new"
  "await"
  "in"
  "yield"
  "get"
  "set"
  "when"
  "out"
  "ref"
  "from"
  "where"
  "select"
  "record"
  "init"
  "with"
  "let"
] @keyword

; pattern expression keywords
(negated_pattern
  "not" @keyword)

(and_pattern
  "and" @keyword)

(or_pattern
  "or" @keyword)

; scoped keyword
(scoped_type
  "scoped" @keyword)

; Attribute
(attribute
  name: (identifier) @attribute)

; Parameters
(parameter
  name: (identifier) @variable.parameter)

; Type constraints
(type_parameter_constraints_clause
  (identifier) @property.definition)

; Method calls
(invocation_expression
  (member_access_expression
    name: (identifier) @function))

; HTML entities (&nbsp; &amp; ...) are mis-parsed by the grammar as C#
; address-of expressions (`& nbsp`); repaint them as strings so they
; match the HTML layer's entity color instead of operator/variable.
; Must stay AFTER the generic (identifier) @variable rule above: for the
; same node the later capture wins, which is what makes this an override.
(razor_implicit_expression
  (prefix_unary_expression
    "&" @string.special
    (identifier) @string.special))

; ----------------------------------------------------------------------
; Overrides — MUST STAY LAST.
;
; For two patterns capturing the SAME node, the later pattern's capture
; wins (verified: tools/highlight-verify). The generic C# token list
; ("{" "}" ... @punctuation.bracket, above) would otherwise paint over
; these.
; ----------------------------------------------------------------------

; @{ ... } block braces — VS Code scopes these explicitly as
; keyword.control.razor.directive.codeblock.open/close.
(razor_block
  "{" @keyword.control
  "}" @keyword.control)

; Control-flow block braces — deliberate deviation from VS Code (which
; leaves these default-colored); requested so every @-construct's braces
; match its transition color.
([
  (razor_if "{" @keyword.control "}" @keyword.control)
  (razor_else "{" @keyword.control "}" @keyword.control)
  (razor_else_if "{" @keyword.control "}" @keyword.control)
  (razor_while "{" @keyword.control "}" @keyword.control)
  (razor_do_while "{" @keyword.control "}" @keyword.control)
  (razor_section "{" @keyword.control "}" @keyword.control)
  (razor_switch "{" @keyword.control "}" @keyword.control)
  (razor_try "{" @keyword.control "}" @keyword.control)
  (razor_catch "{" @keyword.control "}" @keyword.control)
  (razor_finally "{" @keyword.control "}" @keyword.control)
  (razor_lock "{" @keyword.control "}" @keyword.control)
  (razor_foreach "{" @keyword.control "}" @keyword.control)
  (razor_for "{" @keyword.control "}" @keyword.control)
])

; Type-name heuristic (VS Code parity; see reference/README.md): a
; PascalCase identifier on the LEFT of a member access is a type name.
; VS Code gets this from Roslyn semantic tokens; this syntactic
; approximation covers DateTime.ParseExact, CultureInfo.InvariantCulture,
; DateTimeKind.Utc, TimeZoneInfo.Local, etc. Must stay in the overrides
; section: it beats the generic (identifier) @variable on the same node.
((member_access_expression
   expression: (identifier) @type)
  (#match? @type "^[A-Z]"))
