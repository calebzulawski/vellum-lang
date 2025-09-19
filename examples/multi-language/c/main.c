#include <stdio.h>
#include <string.h>
#include "mylibrary.h"

int main(void) {
    // Create store via C ABI
    vellum_owned_ptr_KvStore_ptr store = kv_create();
    kv_set(store.data, "Alice", "teacher");
    kv_set(store.data, "Bob", "musician");
    kv_set(store.data, "Charlie", "chef");
    kv_set(store.data, "Dan", "astronaut");
    kv_delete(store.data, "Dan");

    size_t n = kv_size(store.data);
    printf("%zu entries\n", n);
    const char* alice = kv_get(store.data, "Alice");
    printf("Alice is a %s\n", alice ? alice : "(null)");

    vellum_owned_slice_mut_KvEntry entries = kv_entries(store.data);
    for (size_t i = 0; i < entries.slice_data.len; ++i) {
        struct KvEntry e = entries.slice_data.data[i];
        printf("%s is a %s\n", e.key, e.value);
    }
    // free owned slice and store via deleters
    if (entries.deleter) entries.deleter(entries.slice_data);
    if (store.deleter) store.deleter(store.data);
    return 0;
}
