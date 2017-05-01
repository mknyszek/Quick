func* notOrAll(bits): out {
  foreach b in bits {
    out = b | out;
  }
  !out;
}

func grover(db, item) {
  var qubits = hadamard $ |log2(#db)>;
  var bound = pi/4 * sqrt(N);
  foreach (i in 0..bound) {
    if* (db[qubits] == item) Z(@);
    if* (notOrAll $ hadamard $ qubits) Z(@);
  }
  measure $ qubits
}
