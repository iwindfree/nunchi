; 정의
(function_definition name: (identifier) @name) @def.function
(class_definition name: (identifier) @name) @def.class

; import
(import_from_statement module_name: (dotted_name) @import.path) @import
(import_statement name: (dotted_name) @import.path) @import

; 호출
(call function: (identifier) @callee)
(call function: (attribute attribute: (identifier) @callee))

; 상속
(class_definition name: (identifier) @sub superclasses: (argument_list (identifier) @super))
