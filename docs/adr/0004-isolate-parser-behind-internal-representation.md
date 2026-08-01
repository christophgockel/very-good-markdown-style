# Isolate comrak behind an internal representation

Rules and the rest of the codebase operate on our own internal document representation and a parser interface, never on comrak's types directly. comrak is confined to a single adapter module that implements the interface and maps comrak's AST into ours.

This is the practical follow-through on [ADR 0002](./0002-comrak-parser.md), which noted that swapping parsers would otherwise touch every structural rule. With this boundary, replacing comrak (or adding a second parser) becomes another implementation of the interface rather than a change rippling through every file. The cost is an extra mapping layer and the discipline of not letting comrak types leak past the adapter, which we accept for the isolation it buys.
