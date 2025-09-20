GENERATED := mylibrary.h mylibrary_export.h mylibrary_export.c
EXPORT_FLAGS := -DVELLUM_DYNAMIC

$(GENERATED): ../mylibrary.abi
	cargo run compile c --mode export ../mylibrary.abi

libmylibrary_c.so: $(GENERATED) mylibrary.c
	${CC} -shared -fPIC -fvisibility=hidden $(EXPORT_FLAGS) mylibrary.c mylibrary_export.c -I . -I ../../../library/c++/include -o libmylibrary_c.so

../c++/libmylibrary.so:
	$(MAKE) -C ../c++ libmylibrary.so


test_against_cpp: mylibrary.h ../c++/libmylibrary.so main.c
	${CC} main.c -I . -I ../../../library/c++/include ../c++/libmylibrary.so -Wl,-rpath,'$$ORIGIN/../c++' -o test_c_against_cpp

test_against_c: mylibrary.h libmylibrary_c.so main.c
	${CC} main.c -I . -I ../../../library/c++/include ./libmylibrary_c.so -Wl,-rpath,'$$ORIGIN' -o test_c_against_c

.PHONY: run
run: test_against_cpp test_against_c
	./test_c_against_cpp
	./test_c_against_c

.PHONY: clean
clean:
	rm -f $(GENERATED) test_c_against_cpp test_c_against_c libmylibrary_c.so
