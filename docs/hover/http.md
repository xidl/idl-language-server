# @http

Mark an interface as using the XIDL HTTP mapping.

**Usage** `@http`

**HTTP method annotations**

- `@get(path = "...")`
- `@post(path = "...")`
- `@put(path = "...")`
- `@patch(path = "...")`
- `@delete(path = "...")`
- `@head(path = "...")`
- `@options(path = "...")`
- `@path("...")` (method-level route declaration)

**Media type annotations**

- `@Consumes("mime/type")` (interface or method)
- `@Produces("mime/type")` (interface or method)

**Other annotations**

- `@deprecated`
- `@deprecated("date-or-datetime")`
- `@deprecated(since = "...", after = "...")`

**Parameter annotations**

- `@path` / `@path("name")`
- `@query` / `@query("name")`
- `@header` / `@header("name")`
- `@cookie` / `@cookie("name")`
- `@optional`

**Rules**

- Exactly one verb annotation per method; multiple verbs are invalid.
- If no verb is present, behavior is equivalent to `@post`.
- Route path comes from verb `path` or `@path(...)`.
- Multiple paths are allowed; duplicates after normalization should be
  de-duplicated.
- `@Consumes/@Produces` default to `application/json` and method-level overrides
  interface-level.
- `@head` requires `void` return and only request-side parameters (`in` or
  omitted). `out/inout` are invalid and the response is always `204 No Content`.
