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
var x = not q[0];
var y = not q[1];
print("x = @\n", x);
print("y = @\n", y);
with (a = x and y)
  print("a = @\n", a);
print("x = @\n", x);
print("y = @\n", y);
