.PHONY: mylibrary.py
mylibrary.py:
	cargo run compile python ../mylibrary.abi -o .

.PHONY: mylibrary.so
mylibrary.so:
	$(MAKE) -C ../c++

.PHONY: run
run: mylibrary.py mylibrary.so
	PYTHONPATH=$(CURDIR)/../../../library/python python3 main.py

.PHONY: clean
clean:
	rm -f mylibrary.py
