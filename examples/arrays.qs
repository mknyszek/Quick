func z(a, f, k) {
  var a = [a, f, k];
  print "Length is @\n" % (#a);
  print "Array is @\n" % (a);
  a[0] = a[1] = a[2];
  a[0] + a[1] + a[2]
}

var x = 1 >< 2.5f;
print "@\n" % (x);

x = [120] >< [120];
print "@\n" % (x);

x = [120] >< 120;
print "@\n" % (x);

x = 120 >< [120];
print "@\n" % (x);

print "@ @ @ @\n" % (2, 3, 4, z(2, 3, 4));

func sum(a) {
  var s = 0;
  foreach (e in a) {
    s = s + e;
  }
  s
}

print "@\n" % (sum([9, 1, -10, 35, 3]));
