ABI := ../../mylibrary.abi
RUNTIME_INCLUDE := ../../../../library/c++/include
EXPORT ?= c
OUT := kv_store_from_c_against_$(EXPORT)

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

mylibrary.h: $(ABI)
	cargo run -- compile c $(ABI) -o .

$(OUT): main.c mylibrary.h $(LIB)
	$(CC) main.c -I . -I $(RUNTIME_INCLUDE) $(LIB) $(RPATH) -o $(OUT)

clean:
	rm -f mylibrary.h kv_store_from_c_against_*
