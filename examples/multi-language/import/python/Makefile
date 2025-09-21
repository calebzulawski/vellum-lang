ABI := ../../mylibrary.abi

.PHONY: bindings clean

bindings: mylibrary.py

mylibrary.py: $(ABI)
	cargo run -- compile python $(ABI) -o .

clean:
	rm -f mylibrary.py
