mylibrary.py:
	cargo run compile python ../mylibrary.abi -o .

run: mylibrary.py
	PYTHONPATH=$(CURDIR)/../../../python python3 main.py

clean:
	rm -f mylibrary.py

.PHONY: run clean

