# Multi-language example

This example shows how a single Vellum ABI can be implemented in multiple
languages and then consumed from other languages without changing the call
sites.  The ABI in [`mylibrary.abi`](mylibrary.abi) describes a simple
key-value store.  Shared libraries that satisfy the interface live under
[`export/`](export/) while callers that load those libraries reside under
[`import/`](import/).  Every directory is a self-contained walkthrough of how
to generate bindings, implement the ABI, and exercise the resulting code.

## Running the matrix

All combinations are exercised through pytest.  Install pytest (for example
`pip install pytest`) and run the suite from the repository root:

```bash
pytest examples/multi-language/tests -v
```

Pytest drives the build for every available export and import pairing, compares
each executable's output against
[`tests/data/kv_store_expected.txt`](tests/data/kv_store_expected.txt), and
reports any mismatches.
