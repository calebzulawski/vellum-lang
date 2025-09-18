import os
import platform

import mylibrary


def load_library():
    dirname = os.path.dirname(__file__)
    libname = {
        'Windows': 'mylibrary.dll',
        'Linux': 'libmylibrary.so',
        'Darwin': 'libmylibrary.dylib',
    }[platform.system()]

    return mylibrary.load(os.path.join(dirname, f'../c++/{libname}'))


def main():
    lib = load_library()

    print("kv_create")
    store = lib.kv_create()
    print("kv_set")
    lib.kv_set(store.data, b"Alice", b"teacher")
    print("kv_set")
    lib.kv_set(store.data, b"Bob", b"musician")
    print("kv_set")
    lib.kv_set(store.data, b"Charlie", b"chef")
    print("kv_set")
    lib.kv_set(store.data, b"Dan", b"astronaut")

    lib.kv_delete(store.data, b"Dan")

    print(f"{lib.kv_size(store.data)} entries")

    alice = lib.kv_get(store.data, b"Alice").decode("utf-8")
    print("Alice is a {alice}")

    for entry in lib.kv_entries(store.data):
        key = entry.key.decode("utf-8")
        value = entry.value.decode("utf-8")
        print(f"{key} is a {value}")


if __name__ == "__main__":
    main()
