func fib(n) {
  if (n == 0) 0
  else if (n == 1) 1
  else f(n-1) + f(n-2)
}

print "@\n" % (fib(30));
