; Reuse Zed's HTML highlighting: markup is reparsed with the HTML grammar
; (provided by the html extension), which in turn injects CSS/JS inside
; <style>/<script>.
;
; DO NOT add `injection.combined` here: Zed would merge every matched
; element into ONE HTML layer with multiple included ranges, and Zed's
; query matches elements at several nesting levels (an element that is a
; direct child of a control-flow node can sit inside another element that
; is a direct child of compilation_unit). tree-sitter rejects overlapping
; included ranges ("Incorrect range by index"), which kills the whole HTML
; layer — no markup, CSS or JS highlighting at all. Without `combined`
; each element gets its own layer; nested same-language layers merely
; reparse the same fragment and render identically.

([
  (compilation_unit
    (element) @injection.content)
  (block
    (element) @injection.content)
  (explicit_line_transition
    (element) @injection.content)
  (razor_escape
    (element) @injection.content)
  (razor_block
    (element) @injection.content)
  (razor_catch
    (element) @injection.content)
  (razor_compound_using
    (element) @injection.content)
  (razor_do_while
    (element) @injection.content)
  (razor_else
    (element) @injection.content)
  (razor_else_if
    (element) @injection.content)
  (razor_finally
    (element) @injection.content)
  (razor_if
    (element) @injection.content)
  (razor_lock
    (element) @injection.content)
  (razor_section
    (element) @injection.content)
  (razor_switch_case
    (element) @injection.content)
  (razor_switch_default
    (element) @injection.content)
  (razor_try
    (element) @injection.content)
  (razor_while
    (element) @injection.content)
] (#set! injection.language "html-razor"))
