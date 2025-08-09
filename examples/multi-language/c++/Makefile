mylibrary.hpp:
	cargo run compile cpp ../mylibrary.abi

mylibrary.so: mylibrary.hpp mylibrary.cpp
	${CXX} -shared -fPIC -fvisibility=hidden mylibrary.cpp -I ../../../c++/include -o libmylibrary.so
