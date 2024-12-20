CFLAGS=-std=c11 -g -static

test:
	./tests/test.sh

clean:
	rm -f tmp* *.o

.PHONY: test clean
