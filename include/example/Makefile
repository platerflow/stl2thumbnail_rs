GCC_BIN ?= $(shell which gcc)
CARGO_BIN ?= $(shell which cargo)

run: clean build
	./main
clean:
	rm -f ./main
build:
	$(GCC_BIN) -o ./main ./main.cpp -L. -lstl2thumbnail -lm -ldl -pthread
