.PHONY: mylibrary.hpp
mylibrary.hpp:
	cargo run compile cpp ../mylibrary.abi

libmylibrary.so: mylibrary.hpp mylibrary.cpp
	${CXX} -shared -fPIC -fvisibility=hidden mylibrary.cpp -I ../../../c++/include -o libmylibrary.so

test: main.cpp libmylibrary.so mylibrary.hpp
	${CXX} main.cpp -I ../../../c++/include libmylibrary.so -Wl,-rpath,'$$ORIGIN' -o test

.PHONY: run
run: test
	./test

.PHONY: clean
clean:
	rm -f libmylibrary.so test mylibrary.hpp
