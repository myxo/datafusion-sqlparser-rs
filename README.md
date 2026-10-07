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

# pg_fake_sqlparser

A fork of [Apache DataFusion sqlparser-rs](https://github.com/apache/datafusion-sqlparser-rs),
maintained for `pg_fake`. It carries parser and AST changes needed by the
in-memory PostgreSQL test double. This is an independently maintained fork,
not an official Apache release.

The published package is `pg_fake_sqlparser`; the Rust library name remains
`sqlparser`. It parses SQL into an abstract syntax tree (AST). It does not
execute SQL or validate database schemas.

## Installation

```toml
[dependencies]
sqlparser = { package = "pg_fake_sqlparser", version = "0.63.0" }
```

Keep the dependency key `sqlparser`, including when using the fork's macros.

```rust
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

fn main() -> Result<(), sqlparser::parser::ParserError> {
    let statements = Parser::parse_sql(&PostgreSqlDialect {}, "SELECT 1")?;
    assert_eq!(statements.len(), 1);
    println!("{}", statements[0]);
    Ok(())
}
```

## Features

- `std` and `recursive-protection` are enabled by default.
- `visitor` enables AST traversal and visitor derives.
- `derive-dialect` enables the `derive_dialect!` macro for custom dialects.
- `serde` enables AST serialization and deserialization.

For example, enable AST traversal and custom dialects with:

```toml
sqlparser = { package = "pg_fake_sqlparser", version = "0.63.0", features = ["visitor", "derive-dialect"] }
```

## Parser and derive versions

`pg_fake_sqlparser` **0.63.0** is paired with
`pg_fake_sqlparser_derive` **0.6.0**. The parser pins that derive version exactly
and enables it through the `visitor` or `derive-dialect` features. Most users
should enable those features rather than depend directly on the derive crate.

The `derive_dialect!` macro reads the paired parser's dialect trait source.
It supports the repository's `derive/` layout and Cargo registry directories
named `pg_fake_sqlparser-0.63.0`. Unversioned vendor directories are not
supported by this lookup.

When preparing a new parser version, update `derive/parser-version`, release
an appropriately versioned derive crate, and update the parser's exact derive
dependency together. The version-pairing test checks that the recorded parser
version matches the parser manifest. Fork release numbers and compatibility
are managed independently of upstream.

## Documentation and development

- [Parser API](https://docs.rs/pg_fake_sqlparser/latest/sqlparser/)
- [Derive crate](https://github.com/myxo/datafusion-sqlparser-rs/tree/main/derive)
- [Fork source and issues](https://github.com/myxo/datafusion-sqlparser-rs)
- [Upstream project documentation](https://github.com/apache/datafusion-sqlparser-rs#readme)

Run the parser and derive tests, formatting check, and lints from the repository root:

```sh
cargo test -p pg_fake_sqlparser -p pg_fake_sqlparser_derive --all-features
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## License and attribution

Licensed under [Apache-2.0](LICENSE.TXT). Upstream attribution is retained in
[NOTICE.TXT](NOTICE.TXT) and the source files. This fork includes modifications
for `pg_fake`; upstream authorship is preserved.
