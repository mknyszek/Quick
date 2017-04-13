func hi() {
  print "Hello world\n"; false
}

func bye() {
  print "Goodbye for now world\n"; false
}

var funcs = hi >< bye;
(funcs[0])();
(funcs[1])();
