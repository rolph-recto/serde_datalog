# Serde Datalog - Changelog

## Version 0.2.0 - June 30, 2024

### Changed

- `DatalogExtractor` now assumes ownership of its backend. After serializing
  data with the extractor, you can access the backend with method `get_backend`.

- Added `backend::vector::BackendData<K>` to allow access to generated facts
  stored in hash tables. Can access the vector backend's generated facts with
  new `get_data` method. Previously, the vector backend allows access to facts
  in public `Vec` fields of the backend.

- When storing ints in backends, return error when coercion from unsigned to
  signed ints causes overflow. Previously, overflow is handled by wrapping
  silently, which is the default behavior in release builds.

- Made dependency on `rand` package optional; it is now gated by the `json` feature.
  It is only used for fuzzing the extractor with arbitrary JSON data.

### Added

- Added `DatalogExtractorBackend` impls that assume map keys are always strings,
  which is true for common data formats like JSON and TOML: `StringKeyBackend`
  in `backend::vector` and `backend::souffle_sqlite`.
    - These backends generate facts in a simpler schema (the `key` column of the
      `map` table has type symbol, instead of element ID), which makes writing
      queries against the generated database a bit more convenient. Instead of
      writing `map(elem, key, value), string(key, "key")` you can now write
      `map(elem, "key", value)`.

- Allow multiple input files in `serde_datalog` commandline tool
    - The tool assumes that all files are the same format.
    - `DatalogExtractor` calls `add_root_elem(file, elem)` of backends to
      set `elem` the root elements of input file `file`.

## Version 0.1.1 - June 15, 2024

- Changed license field in Cargo.toml so it shows MIT in crates.io

## Version 0.1.0 - June 15, 2024

Initial release, contains:

- Vector and Souffle SQLite backends
- Support for processing JSON, RON, TOML, and YAML files for `serde_datalog` command line