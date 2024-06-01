# mopper

![](mopper.png)

A fast, lightweight data-to-RDF mapping tool.
It executes an [AlgeMapLoom](https://github.com/s-minoo/algemaploom-rs/blob/main/README.md) mapping plan.

This very early experimantal version takes a mapping plan file in JSON format
as imput and generates RDF as N-Triples or N-Quads.

## Running

Most basically:
```
mopper -m my-mapping-file.json
```

To check all options, run `mopper --help`
```
Usage: mopper [OPTIONS] --mapping-file <FILE>

Options:
  -m, --mapping-file <FILE>  the path to the AlgeMapLoom mapping plan (JSON)
  -v, --verbose...           increase log level
  -q, --quiet                be quiet; no logging
  -h, --help                 Print help
```

## Building
You need Rust and Cargo to build mopper ([install instructions](https://www.rust-lang.org/tools/install)).

Then, in the root directory, run

```
cargo build --release
```

The executable binary comes in the `target/release` directory.


## Current state

Mopper is work in progress. Here's a rough overview of what's (not) implemented:

Input formats: 
- [x] CSV
- [ ] JSON
- [ ] XML

Input / output types:
- [x] File (only input)
- [x] Standard out
- [ ] Standard in
- [ ] Stream (e.g. Kafka, Websocket)
- [ ] Relational database

Output formats:
- [x] N-Triples
- [x] N-Quads
- [ ] More RDF serializations

Mapping features:
- [x] IRI generation function
- [x] Reference function
- [x] IRI template function
- [x] Constant IRI generation
- [x] URL encode function
- [x] IRI generation
- [x] Projection operator
- [x] Fragmenting
- [x] Join operator (only inner join)
- [ ] Blank node generation function
- [ ] Concatenate function
- [ ] Replace function
- [ ] To uppercase  / lowercase function
- [ ] FnO function handling
- [ ] Rename operator
