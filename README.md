# mopper

> I *hate* mappings!*

![](mopper.png)

(*)*That's why mopper tries to do the job as quick as possible!*

A fast and lightweight data-to-RDF mapping tool.
It executes an [AlgeMapLoom](https://github.com/s-minoo/algemaploom-rs/blob/main/README.md) mapping plan which,
in turn, can be generated from an [RML](https://rml.io/)
or [ShExML](https://shexml.herminiogarcia.com/) mappings.

This very early experimental version takes a mapping plan file in JSON format
as input and generates RDF as N-Triples or N-Quads.
Starting from an RML or ShExML mapping directly is on the roadmap.

Conceptually every operator runs in its own thread, and data flow between
them as a stream of messages (as a kind of simplified actor model).
There is still plenty of room for optimizations though...

## Running

Most basically:
```
mopper -m my-mapping-file.json
```

To check all options, run `mopper --help`
```
Usage: mopper [OPTIONS] --mapping-file <FILE>

Options:
  -m, --mapping-file <FILE>          The path to the AlgeMapLoom mapping plan (JSON)
  -v, --verbose...                   Increase log level
  -q, --quiet                        Be quiet; no logging
      --force-std-out                Force output to standard out, ignoring the targets in the plan. Takes precedence over --force-to-file
      --force-to-file <FILE>         Force output to file, ignoring the targets in the plan
      --message-buffer-capacity <N>  Set the maximum number of messages each communication channel can hold before blocking the sender thread. `0` means no messages are hold: 'send' and 'receive' must happen at the same time. The default is `128`
  -d, --deduplicate                  Remove duplicate triples or quads. Note that currently deduplication only works on a per-sink basis and has a negative impact on speed and memory consumption
  -h, --help                         Print help
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
- [x] File
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
- [x] Join operator (only inner join with `equals` condition)
- [x] Blank node generation function
- [x] Deduplication
- [ ] Concatenate function
- [ ] Replace function
- [ ] To uppercase  / lowercase function
- [ ] FnO function handling
- [ ] Rename operator
