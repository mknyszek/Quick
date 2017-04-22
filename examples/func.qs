func hi() {
  print("Hello world\n"); false
}

func what(n) {
  var i = 0;
  while (i < n) {
    print("what");
    i = i + 1;
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


