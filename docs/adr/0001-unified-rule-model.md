# Unified rule model for linting and formatting

We model linting and formatting as one registry of rules rather than two engines or a whole-document reprinter.
Each rule has a detector and an optional fixer. `lint` runs the detectors and reports violations; `format` applies the fixers in place.

We chose this over a Prettier-style reprinter because our primary goal is helpful, targeted messages that explain _why_ a specific location is flagged, which a reprint-the-whole-AST approach cannot give.
We chose it over two independent engines to keep detection and fixing as a single source of truth per rule, so they can never disagree about what "correct" means.
