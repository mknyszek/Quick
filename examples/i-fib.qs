func fib(n) {
  var a = 1;
  var b = 0;
  var i = 0;
  while (i <= n-1) {
    var t = b;
    b = a;
    a = a + t;
    i = i + 1;
  }
  b
}

print "@\n" % (fib(30));
