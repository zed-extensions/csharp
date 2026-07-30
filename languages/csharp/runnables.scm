; A test method: xUnit `[Fact]`/`[Theory]`, NUnit `[Test]`, MSTest `[TestMethod]`.
((method_declaration
  (attribute_list
    (attribute
      name: (identifier) @_attribute))
  name: (_) @run)
  (#match? @_attribute "^(Fact|Theory|Test|TestMethod|TestCase)$")
  (#set! tag csharp-test))

; A test class declared as such: MSTest `[TestClass]`, NUnit `[TestFixture]`.
((class_declaration
  (attribute_list
    (attribute
      name: (identifier) @_attribute))
  name: (_) @run)
  (#match? @_attribute "^(TestClass|TestFixture)$")
  (#set! tag csharp-test-class))

; A test class recognised by its contents. xUnit marks no class-level
; attribute, so the only way to spot one is that it holds a test method.
((class_declaration
  name: (_) @run
  body: (declaration_list
    (method_declaration
      (attribute_list
        (attribute
          name: (identifier) @_attribute)))))
  (#match? @_attribute "^(Fact|Theory|Test|TestMethod|TestCase)$")
  (#set! tag csharp-test-class))

; The entry point of an executable project.
((method_declaration
  name: (_) @run)
  (#eq? @run "Main")
  (#set! tag csharp-main))
