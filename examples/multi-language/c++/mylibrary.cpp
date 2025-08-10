#define VELLUM_EXPORT
#define VELLUM_DYNAMIC

#include <vellum.hpp>
#include "mylibrary.hpp"

#include <map>
#include <memory>
#include <string>

struct KvStore {
    std::map<std::string, std::string> map;
};

namespace vellum_implement {
    vellum::owned<KvStore *> kv_create() noexcept {
        return std::make_unique<KvStore>();
    }

    void kv_set(KvStore *store, const char *key, const char *value) noexcept {
        store->map.emplace(std::make_pair(key, value));
    }

    const char *kv_get(const KvStore *store, const char *key) noexcept {
        if (store->map.count(key)) {
            return store->map.at(key).c_str();
        } else {
            return nullptr;
        }
    }

    void kv_delete(KvStore *store, const char *key) noexcept {
        store->map.erase(key);
    }

    std::size_t kv_size(const KvStore *store) noexcept {
        return store->map.size();
    }

    vellum::owned_slice<KvEntry> kv_entries(const KvStore *store) noexcept {
        std::vector<KvEntry> entries;
        for (const auto& kv : store->map) {
            entries.emplace_back();
            entries.back().key = kv.first.c_str();
            entries.back().value = kv.second.c_str();
        }
        return entries;
    }

    void kv_clear(KvStore *store) noexcept {
        store->map.clear();
    }
}

VELLUM_IMPLEMENT()
