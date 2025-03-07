#!/bin/bash
assert() {
	expected="$1"
	input="$2"

	cargo run -- "$input" > tmp.s
	cc -c tests/sum.c -o sum.o
	cc -c tests/alloc4.c -o alloc4.o
	cc tmp.s sum.o alloc4.o -o tmp -Wa,--noexecstack
	./tmp
	actual="$?"

	if [ "$actual" = "$expected" ]; then
		echo "$input => $actual"
	else
		echo "$input => $expected expected, but got $actual"
		exit 1
	fi
}

assert 0 "int main(){0;}"
assert 42 "int main(){42;}"
assert 21 "int main(){5+20-4;}"
assert 41 "int main(){ 12 + 34 - 5 ;}"
assert 47 'int main(){5+6 * 7;}'
assert 15 'int main(){5*(9-6);}'
assert 4 'int main(){(3+ 5)/2;}'
assert 10 'int main(){-10+20;}'

assert 0 'int main(){0==1;}'
assert 1 'int main(){42==42;}'
assert 1 'int main(){0!=1;}'
assert 0 'int main(){42!=42;}'

assert 1 'int main(){0<1;}'
assert 0 'int main(){1<1;}'
assert 0 'int main(){2<1;}'
assert 1 'int main(){0<=1;}'
assert 1 'int main(){1<=1;}'
assert 0 'int main(){2<=1;}'

assert 1 'int main(){1>0;}'
assert 0 'int main(){1>1;}'
assert 0 'int main(){1>2;}'
assert 1 'int main(){1>=0;}'
assert 1 'int main(){1>=1;}'
assert 0 'int main(){1>=2;}'

assert 3 'int main(){int a;a=3;return 3;}'
assert 8 'int main(){int a;a=3;int b;b=5;return a+b;}'
assert 8 'int main(){int foo;foo=3;int bar;bar=5;return foo+bar;}'
assert 1 'int main(){int foo;foo=3;int bar;bar=5;foo=6;return foo-bar;}'

assert 3 'int main(){if (0) return 2; return 3;}'
assert 3 'int main(){if (1-1) return 2; return 3;}'
assert 2 'int main(){if (1) return 2; return 3;}'

assert 3 'int main(){if (0) return 2; else return 3;}'
assert 2 'int main(){if (1) return 2; else if (0) return 3; else return 4;}'
assert 3 'int main(){if (0) return 2; else if (1) return 3; else return 4;}'
assert 2 'int main(){if (1) return 2; else if (1) return 3; else return 4;}'
assert 4 'int main(){if (0) return 2; else if (0) return 3; else return 4;}'

assert 10 'int main(){int i;i=0;while(i<10) i=i+1; return i;}'

assert 15 'int main(){int sum;int i;sum=0;for(i=0;i<=5;i=i+1)sum=sum+i; return sum;}'
assert 10 'int main(){int sum;int i;sum=0;for(i=0;i<5;i=i+1)sum=sum+i; return sum;}'
assert 10 'int main(){int sum;sum=0;int i; i=0;for(;i<5;i=i+1)sum=sum+i; return sum;}'
assert 5 'int main(){int i;i=0;for(;i<5;)i=i+1; return i;}'

assert 5 'int main(){int sum;sum=0;while(sum < 10){sum=sum+5;return sum;} return sum;}'
assert 10 'int main(){int sum;sum=0;while(sum < 10){sum=sum+5;} return sum;}'
assert 10 'int main(){int sum;sum=0;if(sum < 5) {sum = sum+5; if (sum != 5 ) {sum = 1; return sum;} else if (sum == 5) {sum = 10;}} return sum;}'

assert 3 'int main(){int a;a=1; int b;b=2;int c; c=sum(a, b); return c;}'
assert 5 'int main(){int a;a=1; int b;b=2; int c;c=sum(a*b, sum(a, b)); return c;}'

assert 21 'int fib(int i){if(i==1)return 0;else if(i==2)return 1;return fib(i-1)+fib(i-2);} int main(){int a;a=fib(9); return a;}'
assert 13 'int test(int i){int hoge;hoge = 6+i;return hoge;} int main(){int a;a=test(3); return a+4;}'
assert 19 'int test(int i, int j){int hoge;hoge = 6+i; hoge = hoge+j; return hoge;} int main(){int a;a=test(3, 6); return a+4;}'

assert 3 'int main(){int x;x = 3;int *y; y = &x;return *y;}'

assert 3 'int main(){int x;int *y;y = &x;*y = 3;return x;}'
assert 3 'int main(){int x;int *y;int **z;y = &x;z = &y;**z = 3;return x;}'
assert 5 'int update(int *x){*x = 5;return 0;} int main(){int x;x = 3;update(&x);return x;}'

assert 4 'int main(){int *p;alloc4(&p, 1, 2, 4, 8);int *q;q = p + 2;return *q;}'
assert 8 'int main(){int *p;alloc4(&p, 1, 2, 4, 8);int *q;q = p + 3;return *q;}'
assert 4 'int main(){int *p;alloc4(&p, 1, 2, 4, 8);int *q;q = p + 3;q = q - 1;return *q;}'
assert 4 'int main(){int **p;int *q;alloc4(&q, 1, 2, 4, 8);p = &q;int *r; r = *p + 2;return *r;}'
assert 1 'int main(){int **p;int *q;alloc4(&q, 1, 2, 4, 8);p = &q; return **p;}'
assert 4 'int main(){int **p;int *q;alloc4(&q, 1, 2, 4, 8);p = &q; *p = *p + 2; return **p;}'
assert 8 'int main(){int **p;int *q;alloc4(&q, 1, 2, 4, 8);p = &q; *p = *p + 2; q = q + 1; return **p;}'

assert 4 'int main(){int a;int *b;return sizeof(a);}'
assert 8 'int main(){int a;int *b;return sizeof(b);}'
assert 4 'int main(){int a;int *b;return sizeof(a+4);}'
assert 8 'int main(){int a;int *b;return sizeof(b+4);}'
assert 4 'int main(){int a;int *b;return sizeof(*b);}'
assert 4 'int main(){int a;int *b;return sizeof(1);}'
assert 4 'int main(){int a;int *b;return sizeof(sizeof(1));}'

assert 3 'int main(){int a[3];*a = 3;return *a;}'
assert 3 'int main(){int a[3];int *p;p = a;*p = 3;return *p;}'
assert 2 'int main(){int a[3];*a = 3;*(a+1) = 2;return *(a+1);}'
assert 2 'int main(){int a[3];*a = 3;*(a+1) = 2;int *p;p=a;return *(p+1);}'
assert 3 'int main(){int a[3];*a = 1;*(a+1) = 2;return *a + *(a+1);}'

assert 3 'int main(){int a[3];a[0] = 3;return a[0];}'
assert 2 'int main(){int a[3];a[0] = 3;return a[0]-1;}'
assert 4 'int main(){int a[3];a[0] = 3;a[2] = 1;return a[0]+a[2];}'
assert 3 'int main(){int a[3];int b;b=1;a[b] = 3;return a[1];}'
assert 5 'int main(){int a[3];int b;b=1;a[4-3]=2;a[b+1] = 3;return a[2]+a[1];}'

assert 3 'int a; int main(){a = 3; return a;}'
assert 3 'int a; int main(){a = 3; int b; b = a; return b;}'
assert 5 'int a; int add(int b){return a+b;} int main(){a = 3; return add(2);}'
assert 5 'int a[3]; int add(int b){return a[0]+b;} int main(){a[0] = 3; return add(2);}'
assert 5 'int a[3]; int add(int b){return a[1]+b;} int main(){a[1] = 3; int *p;p=a;return add(2);}'

echo OK
