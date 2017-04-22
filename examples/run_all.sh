#!/bin/sh

run_example() {
  echo "[Example] Running $1"
  target/debug/qscript < examples/$1.qs
}

run_example math
run_example func
run_example i-fib
run_example r-fib
run_example arrays
run_example quantum
