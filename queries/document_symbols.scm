(struct_def (identifier) @struct.name) @struct
(struct_forward_dcl (identifier) @struct.name) @struct

(enum_dcl (identifier) @enum.name) @enum

(bitmask_dcl (identifier) @bitmask.name) @bitmask

(interface_def
  (interface_header (identifier) @interface.name)) @interface
(interface_forward_dcl (identifier) @interface.name) @interface

(op_dcl (identifier) @op.name) @op
