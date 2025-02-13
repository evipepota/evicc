#!/bin/bash
assert() {
	expected="$1"
	input="$2"

	cargo run -- "$input" > tmp.s
	cc -c tests/sum.c -o sum.o
	cc tmp.s sum.o -o tmp
	./tmp
	actual="$?"

	if [ "$actual" = "$expected" ]; then
		echo "$input => $actual"
	else
		echo "$input => $expected expected, but got $actual"
		exit 1
	fi
}

assert 0 "main(){0;}"
assert 42 "main(){42;}"
assert 21 "main(){5+20-4;}"
assert 41 "main(){ 12 + 34 - 5 ;}"
assert 47 'main(){5+6 * 7;}'
assert 15 'main(){5*(9-6);}'
assert 4 'main(){(3+ 5)/2;}'
assert 10 'main(){-10+20;}'

assert 0 'main(){0==1;}'
assert 1 'main(){42==42;}'
assert 1 'main(){0!=1;}'
assert 0 'main(){42!=42;}'

assert 1 'main(){0<1;}'
assert 0 'main(){1<1;}'
assert 0 'main(){2<1;}'
assert 1 'main(){0<=1;}'
assert 1 'main(){1<=1;}'
assert 0 'main(){2<=1;}'

assert 1 'main(){1>0;}'
assert 0 'main(){1>1;}'
assert 0 'main(){1>2;}'
assert 1 'main(){1>=0;}'
assert 1 'main(){1>=1;}'
assert 0 'main(){1>=2;}'

assert 3 'main(){a=3;return 3;}'
assert 8 'main(){a=3;b=5;return a+b;}'
assert 8 'main(){foo=3;bar=5;return foo+bar;}'
assert 1 'main(){foo=3;bar=5;foo=6;return foo-bar;}'

assert 3 'main(){if (0) return 2; return 3;}'
assert 3 'main(){if (1-1) return 2; return 3;}'
assert 2 'main(){if (1) return 2; return 3;}'

assert 3 'main(){if (0) return 2; else return 3;}'
assert 2 'main(){if (1) return 2; else if (0) return 3; else return 4;}'
assert 3 'main(){if (0) return 2; else if (1) return 3; else return 4;}'
assert 2 'main(){if (1) return 2; else if (1) return 3; else return 4;}'
assert 4 'main(){if (0) return 2; else if (0) return 3; else return 4;}'

assert 10 'main(){i=0;while(i<10) i=i+1; return i;}'

assert 15 'main(){sum=0;for(i=0;i<=5;i=i+1)sum=sum+i; return sum;}'
assert 10 'main(){sum=0;for(i=0;i<5;i=i+1)sum=sum+i; return sum;}'
assert 10 'main(){sum=0;i=0;for(;i<5;i=i+1)sum=sum+i; return sum;}'
assert 5 'main(){i=0;for(;i<5;)i=i+1; return i;}'

assert 5 'main(){sum=0;while(sum < 10){sum=sum+5;return sum;} return sum;}'
assert 10 'main(){sum=0;while(sum < 10){sum=sum+5;} return sum;}'
assert 10 'main(){sum=0;if(sum < 5) {sum = sum+5; if (sum != 5 ) {sum = 1; return sum;} else if (sum == 5) {sum = 10;}} return sum;}'

assert 3 'main(){a=1; b=2; c=sum(a, b); return c;}'
assert 5 'main(){a=1; b=2; c=sum(a*b, sum(a, b)); return c;}'

assert 21 'fib(i){if(i==1)return 0;else if(i==2)return 1;return fib(i-1)+fib(i-2);} main(){a=fib(9); return a;}'
assert 13 'test(i){hoge = 6+i;return hoge;} main(){a=test(3); return a+4;}'
assert 19 'test(i, j){hoge = 6+i; hoge = hoge+j; return hoge;} main(){a=test(3, 6); return a+4;}'

assert 3 'main(){x = 3;y = &x;return *y;}'
assert 3 'main(){x = 3;y = 5;z = &y + 8;return *z;}'

echo OK

