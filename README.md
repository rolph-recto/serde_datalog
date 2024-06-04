[![Rust](https://github.com/rolph-recto/serde_datalog/actions/workflows/rust.yml/badge.svg)](https://github.com/rolph-recto/serde_datalog/actions/workflows/rust.yml)

# Serde Datalog

Serde Datalog is a universal extractor of Datalog facts. It implements the
`Serializer` trait from [Serde](https://serde.rs/) to generate a set of
fact tables from any data structure whose type implements the `Serializable`
trait.

## Backends

Currently, Serde Datalog can generate a [SQLite](https://www.sqlite.org/)
database of facts in the format expected by [Souffle](https://souffle-lang.github.io/),
a high-performance implementation of Datalog.
