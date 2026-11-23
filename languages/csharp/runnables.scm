; The entry point of an executable project.
((method_declaration
  name: (_) @run)
  (#eq? @run "Main")
  (#set! tag csharp-main))

; A file of top-level statements, which is the entry point instead of `Main`.
; The button goes on the file's first node; Zed requires `@run` to be small,
; so it can't be the whole file.
((compilation_unit
  .
  (_) @run
  (global_statement))
  (#set! tag csharp-main))

((compilation_unit
  .
  (global_statement) @run)
  (#set! tag csharp-main))
