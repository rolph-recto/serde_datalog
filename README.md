[![Rust](https://github.com/rolph-recto/serde_datalog/actions/workflows/rust.yml/badge.svg)](https://github.com/rolph-recto/serde_datalog/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/serde_datalog?color=blue)](https://crates.io/crates/serde_datalog)
[![docs.rs](https://img.shields.io/docsrs/serde_datalog)](https://docs.rs/serde_datalog/latest/serde_datalog/)

# Serde Datalog

Serde Datalog provides an implementation of the `Serializer` trait from
[Serde](https://serde.rs/) to generate facts from any data structure whose type
implements the `serde::Serializable` trait. In Datalog parlance, Serde Datalog
serializes data structures to EDBs.

Serde Datalog has two main components: an **extractor** that generates facts
about data structures, and a **backend** that materializes these facts into
an explicit representation. You can swap out different implementations of the
backend to change the representation of facts.

# Example

Consider the following enum type that implements the `Serialize` trait:

```rust
#[derive(Serialize)]
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
extractor backend.

For each fact, the extractor will make calls to an extractor backend 
to materialize the fact. For example, we can use the vector backend to
materialize these extracted facts as vectors of tuples.
You can then use these vectors as inputs to queries for Datalog engines embedded
in Rust, such as [Ascent](https://crates.io/crates/ascent) or
[Crepe](https://docs.rs/crepe/latest/crepe/).

```rust
let input = Foo::A(Box::new(Foo::B(10)));
let mut backend = backend::vector::Backend::default();
let mut extractor = DatalogExtractor::new(&mut backend);
input.serialize(&mut extractor);

// Now we can inspect the tables in the backend to see what facts got
// extracted from the input.

// there are 3 total elements
assert!(backend.type_table.len() == 3);

// there are 2 enum variant elements
assert!(backend.variant_type_table.len() == 2);

// there is 1 number element
assert!(backend.number_table.len() == 1);
```

Alternatively, you can store the generated facts in a [SQLite](https://sqlite)
file with the Souffle SQLite backend. You can then use this file as an input
EDB for Datalog queries executed by [Souffle](https://souffle-lang.github.io).

```rust
let input = Foo::A(Box::new(Foo::B(10)));
let mut backend = backend::souffle_sqlite::Backend::default();
let mut extractor = DatalogExtractor::new(&mut backend);
input.serialize(&mut extractor);
backend.dump_to_db("input.db");
```

## Command-line Tool

Serde Datalog also comes as a command-line tool `serde_datalog` that can convert
data from a variety of input formats such as JSON or YAML to a SQLite file
using the Souffle SQLite backend. This allows you to use Souffle Datalog as a
query language for data formats, much like [jq](https://jqlang.github.io/jq/)
or [yq](https://mikefarah.gitbook.io/yq).

### Example

Consider the following JSON file `census.json` containing borough-level
population data in New York City from the 2020 census:

```json
{
	"boroughs": [
		{
			"name": "Bronx",
			"population": 1472654
		},
		{
			"name": "Brooklyn",
			"population": 2736074
		},
		{
			"name": "Manhattan",
			"population": 1694251
		},
		{
			"name": "Queens",
			"population": 2405464
		},
		{
			"name": "Staten Island",
			"population": 495747
		}
	]
}
```

We can write a Souffle Datalog query to calculate the total population of
New York City. First, extract a fact database from the JSON file using
the following invocation of `serde_datalog`:

```
> serde_datalog census.json -o census.db
```

Next, we write the actual query in a Souffle Datalog file, `census.dl`:

```
#include "schemas/serde.dl"

.decl boroPopulation(boro: ElemId, population: number)
boroPopulation(boro, population) :-
    rootElem(root),
    map(root, boroSym, boroList),
    string(boroSym, "boroughs"),
    seq(boroList, _, boro),
    map(boro, popSym, popId),
    string(popSym, "population"),
    number(popId, population).

.decl totalPopulation(total: number)
totalPopulation(sum pop : { boroPopulation(_, pop) }).

.input type, bool, number, string, map, struct, seq, tuple, structType, variantType(IO=sqlite, dbname="census.db")
.output totalPopulation(IO=stdout)
```

Note that the file `schemas/serde.dl` that was included in this query contains
the schema of the input database generated by `serde_datalog`, which
defines the relations `map`, `string`, `seq`, and `number`. For convenience,
the file also defines the relation `rootElem` that returns the root element of
the JSON file.

If we run pass this query to Souffle, we get the following output:

```
> souffle census.dl

---------------
totalPopulation
===============
8804190
===============
```

Note that the schema in `serde.dl` directly represents Serde's data model.
Thus the relations in the schema might not capture all invariants of the
input data format. For instance, maps or records in JSON can only have strings
as keys, whereas the Serde data model allows for non-string keys in maps.

To alleviate this awkward fit, the `schema/` directory contains "overlay"
schemas that make querying specific data formats more natural. For example,
we can rewrite the `boroPopulation` relation above to be shorter using the
JSON overlay schema:

```
#include "schemas/json.dl"

.decl boroPopulation(boro: ElemId, population: number)
boroPopulation(boro, population) :-
    rootElem(root),
    jsonRecord(root, "boroughs", boroList),
    jsonList(boroList, _, boro),
    jsonRecord(boro, "population", popId),
    jsonNumber(popId, population).
```