ABI := ../../mylibrary.abi
RUNTIME_INCLUDE := ../../../../library/c++/include
EXPORT_FLAGS := -DVELLUM_DYNAMIC
GENERATED := mylibrary.hpp mylibrary_export.inl

UNAME_S := $(shell uname -s)

ifeq ($(OS),Windows_NT)
    LIB := mylibrary.dll
else ifeq ($(UNAME_S),Darwin)
    LIB := libmylibrary.dylib
else
    LIB := libmylibrary.so
endif

.PHONY: lib clean

lib: $(LIB)

$(GENERATED): $(ABI)
	cargo run -- compile cpp --mode export $(ABI) -o .

$(LIB): $(GENERATED) mylibrary.cpp
	$(CXX) -shared -fPIC -fvisibility=hidden $(EXPORT_FLAGS) mylibrary.cpp \
		-I $(RUNTIME_INCLUDE) -o $(LIB)

clean:
	rm -f $(GENERATED) $(LIB)
