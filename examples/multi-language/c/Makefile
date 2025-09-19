.PHONY: mylibrary.h
mylibrary.h:
	cargo run compile c ../mylibrary.abi

libmylibrary_c.so: mylibrary.h mylibrary.c
	${CC} -shared -fPIC -fvisibility=hidden mylibrary.c -I . -o libmylibrary_c.so

../c++/libmylibrary.so:
	$(MAKE) -C ../c++ libmylibrary.so

test_against_cpp: mylibrary.h ../c++/libmylibrary.so main.c
	${CC} main.c -I . ../c++/libmylibrary.so -Wl,-rpath,'$$ORIGIN/../c++' -o test_c_against_cpp

test_against_c: mylibrary.h libmylibrary_c.so main.c
	${CC} main.c -I . ./libmylibrary_c.so -Wl,-rpath,'$$ORIGIN' -o test_c_against_c

.PHONY: run
run: test_against_cpp test_against_c
	./test_c_against_cpp
	./test_c_against_c

.PHONY: clean
clean:
	rm -f mylibrary.h test_c_against_cpp test_c_against_c libmylibrary_c.so
