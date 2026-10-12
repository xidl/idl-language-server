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

- `@request(format = "json")` (interface or method)
- `@response(format = "json")` (interface or method)

`format` is one of `json`, `urlencoded`, or `msgpack`.

**Error channels**

- `@http(412)` or `@http(status = 412)` on an `exception` maps it to an HTTP
  error response (status in 300..=599)
- `@http` on a `union` switches on the built-in `ContentType` discriminator for
  content-negotiated responses

**Other annotations**

- `@deprecated`
- `@deprecated("date-or-datetime")`
- `@deprecated(since = "...", after = "...")`
- `@rename("wire_name")` / `@name("wire_name")` on a member
- `@rename_all(rule = "...")` on a container
- `@skip` on a member

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
- `@request`/`@response` default to `application/json` and method-level
  overrides interface-level.
- `@head` requires `void` return and only request-side parameters (`in` or
  omitted). `out/inout` are invalid and the response is always `204 No Content`.
