var a = hadamard $ |3,0>;
print("@\n", #a);
print("@ @ @\n", #a, measure $ a, measure $ hadamard $ hadamard $ |3,0>);

func ftest(f) {
  var y = |2,0>;
  print("\n");
  print("before = @\n", y);
  with (a = f $ y)
    print("during = @\n", a);
  print("after  = @\n", y); 
  false
}

ftest(hadamard);
ftest(sigx);
ftest(sigy);
ftest(sigz);

var q = |2,0b11>;
var x = q[0];
var y = q[1];
print("x = @\n", x);
print("y = @\n", y);
with (a = not (x and y)) {
  print("a = @\n", a);
}
print("x = @\n", x);
print("y = @\n", y);

var qz = |4,0b1110>;
print("before: @\n", qz);
cnot(qz[1:#qz], qz[0]);
print("after: @\n", qz);

var qv = |4, 0b1111>;
print("before: @\n", qv);
with (a = all $ qv[1:#qv]) {
  cflip(a, qv[0]);
}
print("after: @\n", qv);

