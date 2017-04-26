var x = hadamard $ |3>;
print("@\n", #x);
print("@ @ @\n", #x, measure $ x, measure $ hadamard $ hadamard $ |3>);
