import os
import platform

import mylibrary


def load_library(path):
    return mylibrary.load(path)


def run_with(lib):

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


def main():
    dirname = os.path.dirname(__file__)
    libnames = {
        'Windows': ('mylibrary.dll', 'mylibrary_c.dll'),
        'Linux': ('libmylibrary.so', 'libmylibrary_c.so'),
        'Darwin': ('libmylibrary.dylib', 'libmylibrary_c.dylib'),
    }[platform.system()]

    # C++ library
    lib_cpp = load_library(os.path.join(dirname, f'../c++/{libnames[0]}'))
    run_with(lib_cpp)

    # C library
    lib_c = load_library(os.path.join(dirname, f'../c/{libnames[1]}'))
    run_with(lib_c)


if __name__ == "__main__":
    main()
