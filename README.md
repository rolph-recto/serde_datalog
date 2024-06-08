[![Rust](https://github.com/rolph-recto/serde_datalog/actions/workflows/rust.yml/badge.svg)](https://github.com/rolph-recto/serde_datalog/actions/workflows/rust.yml)

# Serde Datalog

Serde Datalog is a universal extractor of Datalog facts. It provides an
implementation of the `Serializer` trait from [Serde](https://serde.rs/)
to generate a set of fact tables from any data structure whose type implements
the `serde::Serializable` trait. In Datalog parlance, Serde Datalog serializes
data structures to EDBs.

Serde Datalog has two main components: an **extractor** that generates facts
about data structures, and a **backend** that materializes these facts into
an explicit representation. You can swap out different implementations of the
backend to change the representation of facts.

# Example

Consider the following enum type:

```
enum Foo {
    A(Box<Foo>),
    B(i64)
}
```

Then consider the enum instance `Foo::A(Foo::B(10))`. The extractor
generates the following facts to represent this data structure:

- Element 1 is a newtype variant
- Element 1 has type `Foo` and variant name `A`
- The first field of Element 1 references Element 2
- Element 2 is a newtype variant
- Element 2 has type `Foo` and variant name `B`
- The first field of Element 2 references Element 3
- Element 3 is an i64
- Element 3 has value 10

The extractor generates facts from a data structure through flattening:
it generates unique identifiers for each element within the data structure,
and references between elements are
["unswizzled"](https://en.wikipedia.org/wiki/Pointer_swizzling)
into identifiers.

For each of these facts, the extractor will make the following calls to an
implementation of [DatalogExtractorBackend]:

```
backend.add_elem(elemId(1), elemType::TupleVariant)
backend.add_variant_type(elemId(1), "Foo", "A")
backend.add_tuple(elemId(1), 0, elemId(2))
backend.add_elem(elemId(2), elemType::TupleVariant)
backend.add_variant_type(elemId(1), "Foo", "B")
backend.add_tuple(elemId(2), 0, elemId(3))
backend.add_elem(elemId(3), elemType::I64)
backend.add_i64(elemId(3), 10)
```

## Backends

Currently, Serde Datalog can generate a [SQLite](https://www.sqlite.org/)
database of facts in the format expected by [Souffle](https://souffle-lang.github.io/),
a high-performance implementation of Datalog.
