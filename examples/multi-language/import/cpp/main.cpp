#define VELLUM_DYNAMIC

#include "mylibrary.hpp"
#include <iostream>

int main() {
    auto store = kv_create().to_unique_ptr();
    kv_set(store.get(), "Alice", "teacher");
    kv_set(store.get(), "Bob", "musician");
    kv_set(store.get(), "Charlie", "chef");
    kv_set(store.get(), "Dan", "astronaut");

    kv_delete(store.get(), "Dan");

    std::cout << kv_size(store.get()) << " entries" << std::endl;
    std::cout << "Alice is a " << kv_get(store.get(), "Alice") << std::endl;

    for (const auto &entry : kv_entries(store.get())) {
        std::cout << entry.key << " is a " << entry.value << std::endl;
    }
}
