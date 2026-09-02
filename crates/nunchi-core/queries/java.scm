; 정의
(class_declaration name: (identifier) @name) @def.class
(interface_declaration name: (identifier) @name) @def.interface
(enum_declaration name: (identifier) @name) @def.enum
(record_declaration name: (identifier) @name) @def.record
(method_declaration name: (identifier) @name) @def.method
(constructor_declaration name: (identifier) @name) @def.constructor
(field_declaration declarator: (variable_declarator name: (identifier) @name)) @def.field

; import
(import_declaration (scoped_identifier) @import.path) @import

; 호출
(method_invocation name: (identifier) @callee)
(object_creation_expression type: (type_identifier) @callee)
