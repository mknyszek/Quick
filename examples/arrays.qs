func z(a, f, k) {
  var a = [a, f, k];
  print("Length is @\n", #a);
  print("Array is @\n", a);
  a[0] = a[1] = a[2];
  a[0] + a[1] + a[2]
}

var x = 1 >< 2.5f;
print("@ = [ 1 2.5f ]\n", x);

x = [120] >< [120];
print("@ = [ 120 120 ]\n", x);

x = [120] >< 120;
print("@ = [ 120 120 ]\n", x);

x = 120 >< [120];
print("@ = [ 120 120 ]\n", x);

x = 120 >< [];
print("@ = [ 120 ]\n", x);

x = cat(1, 2);
print("@ @ @ = [ 1 2 ] 2 1\n", x, len $ x, get(x, 0));
put(x, 0, 3);
print ("@ = [ 3 2 ]\n", x);

print("z(@, @, @) = @ = 12\n", 2, 3, 4, z(2, 3, 4));

func sum(a) {
  var s = 0;
  foreach (e in a) {
    s = s + e;
  }
  s
}

print("@ = 38\n", sum([9, 1, -10, 35, 3]));
