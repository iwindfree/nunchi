; 정의
(function_declaration name: (identifier) @name) @def.function
(class_declaration name: (type_identifier) @name) @def.class
(interface_declaration name: (type_identifier) @name) @def.interface
(enum_declaration name: (identifier) @name) @def.enum
(type_alias_declaration name: (type_identifier) @name) @def.type
(method_definition name: (property_identifier) @name) @def.method

; const Foo = () => {} / function() {} — React 컴포넌트·훅이 대부분 이 형태다
(lexical_declaration
  (variable_declarator
    name: (identifier) @name
    value: [(arrow_function) (function_expression)])) @def.function

; import
(import_statement source: (string) @import.path) @import

; 호출
(call_expression function: (identifier) @callee)
(call_expression function: (member_expression property: (property_identifier) @callee))
