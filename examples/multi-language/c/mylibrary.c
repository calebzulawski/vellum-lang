#include <stdlib.h>
#include <string.h>
#include "mylibrary.h"

struct KvStore {
    struct KvEntry *entries;
    size_t len;
    size_t cap;
};

static void kvstore_free(struct KvStore *s) {
    if (!s) return;
    for (size_t i = 0; i < s->len; ++i) {
        // keys/values are strdup'd
        free((void*)s->entries[i].key);
        free((void*)s->entries[i].value);
    }
    free(s->entries);
    free(s);
}

static void free_kventry_slice(vellum_slice_mut_KvEntry slice) {
    // Only free the backing array allocated in kv_entries
    free(slice.data);
}

vellum_owned_ptr_KvStore_ptr vellum_export_kv_create(void) {
    struct KvStore *s = (struct KvStore*)malloc(sizeof(struct KvStore));
    s->entries = NULL;
    s->len = 0;
    s->cap = 0;
    vellum_owned_ptr_KvStore_ptr out;
    out.data = s;
    out.deleter = (void(*)(struct KvStore*))kvstore_free;
    return out;
}

static char* dup_cstr(const char* s) {
    if (!s) return NULL;
    size_t n = strlen(s);
    char* p = (char*)malloc(n+1);
    if (!p) return NULL;
    memcpy(p, s, n+1);
    return p;
}

void vellum_export_kv_set(struct KvStore *store, const char *key, const char *value) {
    if (!store || !key) return;
    // linear search
    for (size_t i = 0; i < store->len; ++i) {
        if (strcmp(store->entries[i].key, key) == 0) {
            // replace value
            free((void*)store->entries[i].value);
            store->entries[i].value = dup_cstr(value);
            return;
        }
    }
    if (store->len == store->cap) {
        size_t new_cap = store->cap ? store->cap * 2 : 8;
        void* new_mem = realloc(store->entries, new_cap * sizeof(struct KvEntry));
        if (!new_mem) return;
        store->entries = (struct KvEntry*)new_mem;
        store->cap = new_cap;
    }
    store->entries[store->len].key = dup_cstr(key);
    store->entries[store->len].value = dup_cstr(value);
    store->len += 1;
}

const char *vellum_export_kv_get(const struct KvStore *store, const char *key) {
    if (!store || !key) return NULL;
    for (size_t i = 0; i < store->len; ++i) {
        if (strcmp(store->entries[i].key, key) == 0) {
            return store->entries[i].value;
        }
    }
    return NULL;
}

void vellum_export_kv_delete(struct KvStore *store, const char *key) {
    if (!store || !key) return;
    for (size_t i = 0; i < store->len; ++i) {
        if (strcmp(store->entries[i].key, key) == 0) {
            free((void*)store->entries[i].key);
            free((void*)store->entries[i].value);
            // move last into this slot
            if (i + 1 < store->len) {
                store->entries[i] = store->entries[store->len - 1];
            }
            store->len -= 1;
            return;
        }
    }
}

size_t vellum_export_kv_size(const struct KvStore *store) {
    return store ? store->len : 0;
}

vellum_owned_slice_mut_KvEntry vellum_export_kv_entries(const struct KvStore *store) {
    vellum_owned_slice_mut_KvEntry out;
    if (!store || store->len == 0) {
        out.slice_data.data = NULL;
        out.slice_data.len = 0;
        out.deleter = free_kventry_slice;
        return out;
    }
    struct KvEntry* arr = (struct KvEntry*)malloc(store->len * sizeof(struct KvEntry));
    if (!arr) {
        out.slice_data.data = NULL;
        out.slice_data.len = 0;
        out.deleter = free_kventry_slice;
        return out;
    }
    for (size_t i = 0; i < store->len; ++i) {
        arr[i] = store->entries[i];
    }
    out.slice_data.data = arr;
    out.slice_data.len = store->len;
    out.deleter = free_kventry_slice;
    return out;
}

void vellum_export_kv_clear(struct KvStore *store) {
    if (!store) return;
    for (size_t i = 0; i < store->len; ++i) {
        free((void*)store->entries[i].key);
        free((void*)store->entries[i].value);
    }
    store->len = 0;
}

#include "mylibrary_export.inl"
