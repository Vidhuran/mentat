# Datomish

Datomish is a persistent, embedded knowledge base. It's written in ClojureScript, and draws heavily on [DataScript](https://github.com/tonsky/datascript) and [Datomic](http://datomic.com).

Note that at time of writing, there's nothing here.


## Motivation

Datomish is intended to be a flexible relational (not key-value, not document-oriented) store that doesn't leak its storage schema to users, and doesn't make it hard to grow its domain schema and run arbitrary queries.

Our short-term goal is to build a system that, as the basis for a User Agent Service, can support multiple [Tofino](https://github.com/mozilla/tofino) UX experiments without having a storage engineer do significant data migration, schema work, or revving of special-purpose endpoints.


## Comparison to DataScript

DataScript asks the question: "What if creating a database would be as cheap as creating a Hashmap?"

Datomish is not interested in that. Instead, it's strongly interested in persistence and performance, with very little interest in immutable databases/databases as values or throwaway use.

One might say that Datomish's question is: "What if an SQLite database could store arbitrary relations, for arbitrary consumers, without them having to coordinate an up-front storage-level schema?"

(Note that [domain-level schemas are very valuable](http://martinfowler.com/articles/schemaless/).)

Another possible question would be: "What if we could bake some of the concepts of CQRS and event sourcing into a persistent relational store, such that the transaction log itself were of value to queries?"

Some thought has been given to how databases as values — long-term references to a snapshot of the store at an instant in time — could work in this model. It's not impossible; it simply has different performance characteristics.

Just like DataScript, Datomish speaks Datalog for querying and takes additions and retractions as input to a transaction. Unlike DataScript, Datomish's API is asynchronous.

Unlike DataScript, Datomish exposes free-text indexing, thanks to SQLite.


## Comparison to Datomic

Datomic is a server-side, enterprise-grade data storage system. Datomic has a beautiful conceptual model. It's intended to be backed by a storage cluster, in which it keeps index chunks forever. Index chunks are replicated to peers, allowing it to run queries at the edges. Writes are serialized through a transactor.

Many of these design decisions are inapplicable to deployed desktop software; indeed, the use of multiple JVM processes makes Datomic's use in a small desktop app, or a mobile device, prohibitive.

Datomish is designed for embedding, initially in an Electron app ([Tofino](https://github.com/mozilla/tofino)). It is less concerned with exposing consistent database states outside transaction boundaries, because that's less important here, and dropping some of these requirements allows us to leverage SQLite itself.


## Contributing

Please note that this project is released with a Contributor Code of Conduct.
By participating in this project you agree to abide by its terms.

See [CONTRIBUTING.md](/CONTRIBUTING.md) for further notes.

This project is very new, so we'll probably revise these guidelines. Please
comment on a bug before putting significant effort in if you'd like to
contribute.


## License

At present this code is licensed under MPLv2.0. That license is subject to change prior to external contributions.