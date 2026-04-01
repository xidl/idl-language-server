(ERROR) @error

(struct_def (identifier) @type.def)
(struct_forward_dcl (identifier) @type.def)

(enum_dcl (identifier) @type.def)
(bitmask_dcl (identifier) @type.def)

(interface_def (interface_header (identifier) @type.def))
(interface_forward_dcl (identifier) @type.def)

(typedef_dcl
  (type_declarator
    (any_declarators
      (any_declarator (simple_declarator (identifier) @type.def)))))

(typedef_dcl
  (type_declarator
    (any_declarators
      (any_declarator (array_declarator (identifier) @type.def)))))

(simple_type_spec (scoped_name (identifier) @type.ref))
