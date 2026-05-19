; =========================
; Test method with namespace
; =========================
(
  (namespace_declaration
    name: (_) @csharp_namespace
    body: (declaration_list
      (class_declaration
        name: (identifier) @csharp_class_name @csharp_csproj_hint
        body: (declaration_list
          (method_declaration
            (attribute_list
              (attribute
                name: (_) @attribute_name
                (#match? @attribute_name "^(Fact|Theory|Test|TestCase|TestCaseSource|TestMethod|DataTestMethod)$")))
            name: (identifier) @run @csharp_method_name)))))
  (#set! tag csharp-test-method)
)

; ======================================
; Test method with file-scoped namespace
; ======================================
(
  (file_scoped_namespace_declaration
    name: (_) @csharp_namespace
    (class_declaration
      name: (identifier) @csharp_class_name @csharp_csproj_hint
      body: (declaration_list
        (method_declaration
          (attribute_list
            (attribute
              name: (_) @attribute_name
              (#match? @attribute_name "^(Fact|Theory|Test|TestCase|TestCaseSource|TestMethod|DataTestMethod)$")))
          name: (identifier) @run @csharp_method_name))))
  (#set! tag csharp-test-method)
)

; ============================
; Test method without namespace
; (anchored to top-level only)
; ============================
(
  (compilation_unit
    (class_declaration
      name: (identifier) @csharp_class_name @csharp_csproj_hint
      body: (declaration_list
        (method_declaration
          (attribute_list
            (attribute
              name: (_) @attribute_name
              (#match? @attribute_name "^(Fact|Theory|Test|TestCase|TestCaseSource|TestMethod|DataTestMethod)$")))
          name: (identifier) @run @csharp_method_name))))
  (#set! tag csharp-test-method)
)

; ===============================
; Test class with namespace
; ===============================
(
  (namespace_declaration
    name: (_) @csharp_namespace
    body: (declaration_list
      (class_declaration
        (attribute_list
          (attribute
            name: (_) @class_attribute
            (#match? @class_attribute "^(TestFixture|TestClass)$")))
        name: (identifier) @run @csharp_class_name)))
  (#set! tag csharp-test-class)
)

; =====================================
; Test class with file-scoped namespace
; =====================================
(
  (file_scoped_namespace_declaration
    name: (_) @csharp_namespace
    (class_declaration
      (attribute_list
        (attribute
          name: (_) @class_attribute
          (#match? @class_attribute "^(TestFixture|TestClass)$")))
      name: (identifier) @run @csharp_class_name))
  (#set! tag csharp-test-class)
)

; ==============================
; Test class without namespace
; (anchored to top-level only)
; ==============================
(
  (compilation_unit
    (class_declaration
      (attribute_list
        (attribute
          name: (_) @class_attribute
          (#match? @class_attribute "^(TestFixture|TestClass)$")))
      name: (identifier) @run @csharp_class_name))
  (#set! tag csharp-test-class)
)
