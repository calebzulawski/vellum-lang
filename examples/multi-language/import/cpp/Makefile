ABI := ../../mylibrary.abi
RUNTIME_INCLUDE := ../../../../library/c++/include
EXPORT ?= cpp
OUT := kv_store_from_cpp_against_$(EXPORT)

UNAME_S := $(shell uname -s)

ifeq ($(OS),Windows_NT)
    LIB_NAME := mylibrary.dll
    RPATH :=
else ifeq ($(UNAME_S),Darwin)
    LIB_NAME := libmylibrary.dylib
    RPATH := -Wl,-rpath,@loader_path/../../export/$(EXPORT)
else
    LIB_NAME := libmylibrary.so
    RPATH := -Wl,-rpath,'$$ORIGIN/../../export/$(EXPORT)'
endif

LIB_DIR := ../../export/$(EXPORT)
LIB := $(LIB_DIR)/$(LIB_NAME)

.PHONY: binary clean

binary: $(OUT)

mylibrary.hpp: $(ABI)
	cargo run -- compile cpp $(ABI) -o .

$(OUT): main.cpp mylibrary.hpp $(LIB)
	$(CXX) main.cpp -I . -I $(RUNTIME_INCLUDE) $(LIB) $(RPATH) -o $(OUT)

clean:
	rm -f mylibrary.hpp kv_store_from_cpp_against_*
