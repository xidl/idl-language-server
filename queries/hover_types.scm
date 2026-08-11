; Type definitions used for hover: `@def` is the whole declaration, `@def.name`
; is the declared type name.
(struct_def (identifier) @def.name) @def
(struct_forward_dcl (identifier) @def.name) @def
(enum_dcl (identifier) @def.name) @def
(bitmask_dcl (identifier) @def.name) @def
(bitset_dcl (identifier) @def.name) @def
(union_def (identifier) @def.name) @def
(union_forward_dcl (identifier) @def.name) @def
(except_dcl (identifier) @def.name) @def
(native_dcl (simple_declarator (identifier) @def.name)) @def
(interface_def (interface_header (identifier) @def.name)) @def
(interface_forward_dcl (identifier) @def.name) @def
(value_forward_dcl (identifier) @def.name) @def
(value_box_def (identifier) @def.name) @def
(component_forward_dcl (identifier) @def.name) @def
(event_forward_dcl (identifier) @def.name) @def
(porttype_forward_dcl (identifier) @def.name) @def
(typedef_dcl (type_declarator (any_declarators (any_declarator (simple_declarator (identifier) @def.name))))) @def

; Type references: the identifier under a scoped name, or the head of a
; user-defined template application such as `IntList<int32>`.
(scoped_name (identifier) @type.name)
(template_type (identifier) @type.name)

; Builtin type nodes: the whole node is captured so `sequence<int32>` and
; `string<32>` are handled, with the inner type still matched separately.
[
  (signed_tiny_int) (signed_short_int) (signed_long_int) (signed_longlong_int)
  (unsigned_tiny_int) (unsigned_short_int) (unsigned_long_int) (unsigned_longlong_int)
  (boolean_type) (octet_type) (floating_pt_type) (char_type) (wide_char_type)
  (string_type) (wide_string_type) (any_type) (object_type) (value_base_type)
  (fixed_pt_const_type) (fixed_pt_type) (sequence_type) (map_type)
] @builtin.type
