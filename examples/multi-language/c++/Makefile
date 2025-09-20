GENERATED := mylibrary.hpp mylibrary_export.inl
EXPORT_FLAGS := -DVELLUM_DYNAMIC

$(GENERATED): ../mylibrary.abi
	cargo run compile cpp --mode export ../mylibrary.abi

libmylibrary.so: $(GENERATED) mylibrary.cpp
	${CXX} -shared -fPIC -fvisibility=hidden $(EXPORT_FLAGS) mylibrary.cpp -I ../../../library/c++/include -o libmylibrary.so

test: main.cpp libmylibrary.so mylibrary.hpp
	${CXX} main.cpp -I ../../../library/c++/include libmylibrary.so -Wl,-rpath,'$$ORIGIN' -o test

test_against_c: main.cpp mylibrary.hpp ../c/libmylibrary_c.so
	${CXX} main.cpp -I ../../../library/c++/include ../c/libmylibrary_c.so -Wl,-rpath,'$$ORIGIN/../c' -o test_against_c

../c/libmylibrary_c.so:
	$(MAKE) -C ../c libmylibrary_c.so

.PHONY: run
run: test test_against_c
	./test
	./test_against_c

.PHONY: clean
clean:
	rm -f libmylibrary.so test test_against_c $(GENERATED)
