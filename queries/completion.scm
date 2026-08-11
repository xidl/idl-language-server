; Names of user-defined compound types for type completion.
(struct_def (identifier) @type.name)
(struct_forward_dcl (identifier) @type.name)
(union_def (identifier) @type.name)
(union_forward_dcl (identifier) @type.name)
(enum_dcl (identifier) @type.name)
(bitmask_dcl (identifier) @type.name)
(bitset_dcl (identifier) @type.name)
(except_dcl (identifier) @type.name)
(interface_def (interface_header (identifier) @type.name))
(interface_forward_dcl (identifier) @type.name)
(typedef_dcl (type_declarator (any_declarators (any_declarator (simple_declarator (identifier) @type.name)))))

; Custom annotations: definitions and scoped-name usages.
(annotation_dcl (identifier) @annotation.name)
(annotation_appl_custom_body (scoped_name) @annotation.name)
