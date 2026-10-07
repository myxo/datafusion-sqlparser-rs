<!---
  Licensed to the Apache Software Foundation (ASF) under one
  or more contributor license agreements.  See the NOTICE file
  distributed with this work for additional information
  regarding copyright ownership.  The ASF licenses this file
  to you under the Apache License, Version 2.0 (the
  "License"); you may not use this file except in compliance
  with the License.  You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

  Unless required by applicable law or agreed to in writing,
  software distributed under the License is distributed on an
  "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
  KIND, either express or implied.  See the License for the
  specific language governing permissions and limitations
  under the License.
-->

# pg_fake_sqlparser_derive

Procedural macros for the `pg_fake_sqlparser` fork maintained for `pg_fake`.
This package derives from [Apache DataFusion sqlparser-rs](https://github.com/apache/datafusion-sqlparser-rs)
and is not an official Apache release. Its Rust library name remains
`sqlparser_derive`.

It provides `Visit` and `VisitMut` derives for AST traversal and
`derive_dialect!` for custom SQL dialects.

## Recommended use

Enable the corresponding features on the parser crate. It selects the paired
derive version automatically:

```toml
[dependencies]
sqlparser = { package = "pg_fake_sqlparser", version = "0.63.0", features = ["visitor", "derive-dialect"] }
```

```rust
use sqlparser::derive_dialect;
use sqlparser::dialect::{Dialect, GenericDialect};

derive_dialect!(CustomDialect, GenericDialect, overrides = {
    supports_order_by_all = true,
});

fn main() {
    assert!(CustomDialect::new().supports_order_by_all());
}
```

## Direct dependency and version pairing

If you need a direct dependency on the procedural macros, use both aliases:

```toml
[dependencies]
sqlparser = { package = "pg_fake_sqlparser", version = "=0.63.0", features = ["visitor"] }
sqlparser_derive = { package = "pg_fake_sqlparser_derive", version = "=0.6.0" }
```

Keep the `sqlparser` dependency key because generated code refers to that
name. Derive version **0.6.0** is paired with parser version **0.63.0**.
Do not substitute the upstream `sqlparser` or `sqlparser_derive` packages.

For `derive_dialect!`, the shipped `parser-version` file identifies the
parser source version. The macro reads `src/dialect/mod.rs` from the parser
repository above `derive/`, or from the sibling Cargo registry directory
`pg_fake_sqlparser-0.63.0`. It reports an error when that source is missing,
instead of selecting another cached parser version. Unversioned vendor
layouts are not supported.

## Links and license

- [Macro API](https://docs.rs/pg_fake_sqlparser_derive/latest/sqlparser_derive/)
- [Parser API](https://docs.rs/pg_fake_sqlparser/latest/sqlparser/)
- [Fork repository](https://github.com/myxo/datafusion-sqlparser-rs)

Licensed under [Apache-2.0](LICENSE.TXT). Upstream attribution is retained in
[NOTICE.TXT](NOTICE.TXT) and the source files. This fork includes modifications
for `pg_fake`; upstream authorship is preserved.
