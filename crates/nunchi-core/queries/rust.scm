; 정의
(function_item name: (identifier) @name) @def.function
(struct_item name: (type_identifier) @name) @def.struct
(enum_item name: (type_identifier) @name) @def.enum
(trait_item name: (type_identifier) @name) @def.trait
(mod_item name: (identifier) @name) @def.module
(type_item name: (type_identifier) @name) @def.type
(const_item name: (identifier) @name) @def.const

; import
(use_declaration argument: (_) @import.path) @import

; 호출 (빠른 경로 — 이름 기반 해소)
(call_expression function: (identifier) @callee)
(call_expression function: (scoped_identifier name: (identifier) @callee))
(call_expression function: (field_expression field: (field_identifier) @callee))

; 트레이트 구현
(impl_item trait: (type_identifier) @super type: (type_identifier) @sub)
