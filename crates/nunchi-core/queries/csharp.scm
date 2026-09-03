; 정의
(class_declaration name: (identifier) @name) @def.class
(interface_declaration name: (identifier) @name) @def.interface
(struct_declaration name: (identifier) @name) @def.struct
(record_declaration name: (identifier) @name) @def.record
(enum_declaration name: (identifier) @name) @def.enum
(method_declaration name: (identifier) @name) @def.method
(constructor_declaration name: (identifier) @name) @def.constructor
(property_declaration name: (identifier) @name) @def.property

; import
(using_directive (qualified_name) @import.path) @import
(using_directive (identifier) @import.path) @import

; 호출
(invocation_expression function: (identifier) @callee)
(invocation_expression function: (member_access_expression name: (identifier) @callee))
(object_creation_expression type: (identifier) @callee)

; 상속·구현
(class_declaration name: (identifier) @sub (base_list (identifier) @super))
(interface_declaration name: (identifier) @sub (base_list (identifier) @super))
