"""Exercise the generated Python bindings against a compiled export library."""

from __future__ import annotations

import argparse
import platform
from pathlib import Path

import mylibrary

EXAMPLE_ROOT = Path(__file__).resolve().parents[2]
EXPORT_ROOT = EXAMPLE_ROOT / "export"

EXPORT_CHOICES = tuple(
    sorted(
        directory.name
        for directory in EXPORT_ROOT.iterdir()
        if directory.is_dir() and (directory / "Makefile").exists()
    )
)

DEFAULT_EXPORT = "c"


def run_with(lib) -> None:
    """Exercise the key-value store with the provided library handle."""

    store = lib.kv_create()
    lib.kv_set(store.data, b"Alice", b"teacher")
    lib.kv_set(store.data, b"Bob", b"musician")
    lib.kv_set(store.data, b"Charlie", b"chef")
    lib.kv_set(store.data, b"Dan", b"astronaut")

    lib.kv_delete(store.data, b"Dan")

    print(f"{lib.kv_size(store.data)} entries")

    alice = lib.kv_get(store.data, b"Alice").decode("utf-8")
    print(f"Alice is a {alice}")

    for entry in lib.kv_entries(store.data):
        key = entry.key.decode("utf-8")
        value = entry.value.decode("utf-8")
        print(f"{key} is a {value}")


def resolve_library_path(export: str) -> Path:
    """Locate the compiled shared library for ``export``."""

    system = platform.system()
    if system == "Windows":
        library_name = "mylibrary.dll"
    elif system == "Darwin":
        library_name = "libmylibrary.dylib"
    else:
        library_name = "libmylibrary.so"

    return EXPORT_ROOT / export / library_name


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--library",
        choices=EXPORT_CHOICES,
        default=DEFAULT_EXPORT,
        help="Which export implementation to load (default: %(default)s)",
    )
    args = parser.parse_args(argv)

    library = mylibrary.load(str(resolve_library_path(args.library)))
    run_with(library)


if __name__ == "__main__":
    main()
