# Quick Programming Language

A prototype quantum scripting language, complete with a simple bytecode-based
interpreter.

## License

This repository is licensed under GPLv3, because it depends upon
`rust-libquantum`, which is GPLv3 because it ultimately depends on libquantum.

## Installation

`git clone` this repository and run `cargo build --release`.

## Run

Run the command `target/release/qscript < <some file>.qs` to execute a QScript
program.

## Tutorial

Quick derives much of its syntax and semantics from JavaScript. Thus, in this
tutorial, the syntax that is considered "common" will have very little
explanation associated with it.

### Comments

Comments in Quick are pieces of text that are ignored. One denotes a comment
much like in C or JavaScript:

```
// This is an inline-comment
/*
this
is

a multiple
line comment
*/
```

### Numerical Primitives

The Quick programming language supports a signed 64-bit integer type. In the
syntax, it also supports multiple representations for integers.

```
23 // decimal integer 23
0b10111 // binary representation for decimal integer 23
0x17 // hexadecimal representation for decimal integer 23
```

Quick also supports 64-bit floating-point values, denoted with a suffix "f."

```
2.3f // floating-point number 2.3
```

One may perform basic arithmetic operations on these numerical primitives.

```
3 + 4 // = 5
3.4f - 3 // = 0.4
2 * 34 // = 68
10 / 5.0f // = 2.0f
2 ^^ 5 // = 32
5 % 2 // = 1, remainder operator
```

Note that conversions between floating-point numbers and integers are implicit,
and that where necessary, a result will be converted to a floating-point as it
is "more general."



### Booleans

Boolean values are also supported by Quick. Below are the literal values for
"true" and "false":

```
true
false
```

One may also operate on these Boolean values using the logical operators "and,"
"or," and "not":

```
not true // = false
true and false // = false
true or false // = true
```

### Statements and Expressions

A Quick program is made of up of statements. Every statement, with the
exception of the block statement, must end in a semicolon. Statements give no
value as a result. For example, the following Quick program is correct:

```
34;
356;
35;
```

while this one is not

```
34
356
35
```

The numbers "34," "356," and "35" are expressions which give back their
numerical value as a result. Because a valid Quick program is made of up
statements, putting these expressions next to each other is not allowed.

### Variables

An important statement in Quick is the variable definition statement.

One may define new variables in Quick similarly to JavaScript. Note that the
variable must always have an explicit initialization value. In addition,
variables within the same scope (more on this later) may not be redefined.

```
var x = 4; // binds the number 4 to variable x
var y; // error, variables need an initialization value
var x = 5; // error, cannot be redefined
```

Variables may then be used as expressions. For example,

```
var x = 4;
var y = 5;
var z = x + y; // z = 9
```

Note also that there are no global variables in Quick, meaning that variables
defined at the top-level of a program may not be used everywhere, like in other
languages such as JavaScript or Python.

Finally, Quick provides an assignment expression which allows already-defined
variables to be re-assigned. Note that because assignment is an expression, it
always returns the value being assigned, such that assignment expressions may
be chained together.

```
var x = 4; // x is 4
x = 3; // x is now 3
var y = x = 2; // both x and y are now 2
```

### Blocks & Scoping

Quick is a lexically scoped language, meaning variables are only allowed to be
referenced if they are defined in the current scope, or a parent scope. More
specifically, this means that there is some syntax which one can use to
determine what scope a variable belongs to. It is only valid on that scope,
after its definition.

In Quick, scope is defined using the block statement and the block expression. 

```
{ // block statement
  var x = 4;
  x + 5;
}

x + 4; // error, x is no longer defined.

var x = { // block expression
  var y = 4;
  y
};

y + 4; // error, y is no longer defined.
```

Note that the different between a block statement and a block expression
is that the final entry in a block statement has a semi-colon while a block
expression does not (recall: statements end in semi-colons). That lack of
semi-colon in a block expression indicates that the final entry is an
expression, and acts as the value which the block expression evaluates
to.

In order to reduce the number of semi-colons in the code, block statements
do not require semi-colons after them, so the following code is a valid
block statement and thus a valid Quick program:

```
{ // block statement
  { var x = 3; } // block statement
  { var y = 3; } // block statement
}
```

Because this code does not return a value, it is not a block expression,
and is instead a block statement. Finally, note that variables may "shadow"
variables from outer scopes. For example,

```
var x = 3;
var y = {
  var x = 2;
  x + 1
}; // y contains 3, not 4
```

That means, while variable redefinition in the same scope is not allowed,
variable definition in new blocks are allowed.

### Print Statement

In order for a Quick program to have output, a special print statement is
provided. The print statement has the following syntax

```
print("hi"); // prints "hi"
print("@", 3); // prints "3"
var x = 3;
print("@\n", x); // prints "3" with a new-line character
```

The print statement prints the character string that is in quotes, but replaces
any instances of "@" with the corresponding arguments. For this reason, the
print statement can take an arbitrary number of expressions:

```
print("@ @ @ @ @", 3, 4, 5, 6, 7); // prints "3 4 5 6 7"
```

In order to print a "@" character, simply escape it. Below is a list of all the
supported escaped characters in Quick's print statement:

```
print("\n"); // prints a new line character
print("\r"); // prints a carriage return character
print("\t"); // prints a tab character
print("\""); // prints a double-quote character
print("\@"); // prints a "@" character
```

### Functions

Functions in Quick may be defined using the function definition statement:

```
func f() 3;
```

The above code defines a function that takes no arguments and returns the
value "3". The body of a function definition statement MUST be an expression,
meaning that all functions in Quick must return a value. Thus, the following
code is invalid:

```
func f() var x = 3;;
```

Note that functions in Quick are not closures, so they do not capture
surrounding state. Additionally, because there are no global variables,
this means that there is no way to access a variable from outside a function
defintion. For example, the following code is invalid:

```
var x = 3;
func f() x; // error, undefined variable x
```

Here's an example of two equivalent functions in Quick that sum two numbers:

```
func add(x, y) {
  var z = x + y;
  z
}

func add2(x, y) x + y;
```

To actually invoke a function in Quick, one may use the familiar C-style
function call syntax. Note that arguments are expressions that are
comma-separated.

```
var x = add(1, 2); // x contains 3
```

Also, functions in Quick are first-class values. This means that one may
refer to a function and put it into a variable. This variable may now be
used as a function. For example,

```
func f() 3;
var x = f;
var y = x(); // y contains 3
```

Finally, Quick provides two additional ways to call a function, using the
$ character.

```
func f(x) x;
func g() 3;
var x = f $ 3; // this syntax invokes a function with 1 argument
// x now contains 3
var y = $g; // this syntax invokes a function with 0 arguments
// y now contains 3
```

### More coming soon!

## Progress

There are currently numerous bugs, and the language is not yet
feature-complete. The current bugs and missing features may be found in the
issues section on this GitHub page.
