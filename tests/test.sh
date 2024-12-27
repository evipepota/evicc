#!/bin/bash
assert() {
	expected="$1"
	input="$2"

	cargo run -- "$input" > tmp.s
	cc -o tmp tmp.s
	./tmp
	actual="$?"

	if [ "$actual" = "$expected" ]; then
		echo "$input => $actual"
	else
		echo "$input => $expected expected, but got $actual"
		exit 1
	fi
}

assert 0 "0;"
assert 42 "42;"
assert 21 "5+20-4;"
assert 41 " 12 + 34 - 5 ;"
assert 47 '5+6 * 7;'
assert 15 '5*(9-6);'
assert 4 '(3+ 5)/2;'
assert 10 '-10+20;'

assert 0 '0==1;'
assert 1 '42==42;'
assert 1 '0!=1;'
assert 0 '42!=42;'

assert 1 '0<1;'
assert 0 '1<1;'
assert 0 '2<1;'
assert 1 '0<=1;'
assert 1 '1<=1;'
assert 0 '2<=1;'

assert 1 '1>0;'
assert 0 '1>1;'
assert 0 '1>2;'
assert 1 '1>=0;'
assert 1 '1>=1;'
assert 0 '1>=2;'

assert 3 'a=3;return 3;'
assert 8 'a=3;b=5;return a+b;'
assert 8 'foo=3;bar=5;return foo+bar;'
assert 1 'foo=3;bar=5;foo=6;return foo-bar;'

assert 3 'if (0) return 2; return 3;'
assert 3 'if (1-1) return 2; return 3;'
assert 2 'if (1) return 2; return 3;'

assert 3 'if (0) return 2; else return 3;'
assert 2 'if (1) return 2; else if (0) return 3; else return 4;'
assert 3 'if (0) return 2; else if (1) return 3; else return 4;'
assert 2 'if (1) return 2; else if (1) return 3; else return 4;'
assert 4 'if (0) return 2; else if (0) return 3; else return 4;'

assert 10 'i=0;while(i<10) i=i+1; return i;'

assert 15 'sum=0;for(i=0;i<=5;i=i+1)sum=sum+i; return sum;'
assert 10 'sum=0;for(i=0;i<5;i=i+1)sum=sum+i; return sum;'
assert 10 'sum=0;i=0;for(;i<5;i=i+1)sum=sum+i; return sum;'
assert 5 'i=0;for(;i<5;)i=i+1; return i;'

echo OK

