#!/bin/sh

run_example() {
  echo "[Example] Running $1"
  target/debug/quick < examples/$1.qk
}

run_example math
run_example func
run_example i-fib
run_example r-fib
run_example arrays
run_example quantum
run_example teleport
