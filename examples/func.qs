func hi() {
  print("Hello world\n"); false
}

func what(n) {
  for (i in 0..n) {
    print("what");
  }
  print("\n");
  false
}

func bye() {
  print("Goodbye for now world\n"); false
}

var funcs = hi >< bye;
(funcs[0])();
var x = what;
x $ 10;
(funcs[1])();


