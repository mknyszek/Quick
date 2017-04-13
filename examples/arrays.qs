func z(a, f, k) {
  var a = [a, f, k];
  a[0] = a[1] = a[2];
  print "@\n" % (a);
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
