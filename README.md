# sal
sal stands for Simple Assembly-Like, as this language is simple and like an assembly language!
It's a work in progress right now!

---
## values and types in sal:  
Sal has three value types: int, float, and String.
ints and floats are both 64 bit, strings are arbitrary length and can contain any valid utf-8 characters (per Rust's String implementation).  
sal is not strongly typed, and features automatic type coercion whenever possible. If sal's typing rules are invalidated, execution crashes. There are only so many operations in sal, so here are the rules:  
Addition: {value of type 1} + {value of type 2} = {which type?}  
{string} + {int} -> {string}  
{string} + {float} -> {string}  
{string} + {string} -> {string}  
{float} + {float} -> {float}  
{float} + {int} -> {float}  
{int} + {int} -> {int}  
while these rules work for addition, an operation like a string multiplied by a string makes no sense, so multiplying, subtracting, and dividing with strings will cause a crash.  
Numerical operations on these types will work just like outlined in addition: a float times an int will yield a float, but an int times an int, or even an int divided by an int, will yield an int.

---
current supported instructions:
- pushi  
    stands for "push immediate," pushes an immediate/literal value onto the top of the stack.
- pushr  
    stands for "push register," pushes the contents of the A register onto the stack
- pops  
    stands for "pop and save," pops the top value off the stack and places that value into the A register.
- pop  
    Pops the top value off the stack, does not save that value anywhere
- peek  
    places a copy of the top value of the stack into the A register, does not remove any values off the stack.
- jump  
    jumps to executing a different line in the file
- jzer  
    stands for "jump if zero," jumps to executing a different line in the file only if the value 0 is in the A register when this is called.
- call  
    experimental at the moment. Acts like the jump instruction but saves the line number that this instruction is called from to the R register. This is the only way to modify the R register.
- ret  
    experimental at the moment. Jumps to the line number specified by the value in the R register.
- add  
    adds the top two values of the stack together and places the result in the A register.
- mult  
    multiplies the top two values of the stack together and places the result in the A register.
- swap  
    swaps the values of registers A and B. Register B is mostly a convenience to make programming in sal at least a little bearable. 
- inc  
    increments the value in the A register by one. Only works if an integer value is currently in the A register.
- dec  
    decrements the value in the A register by one. Only works if an integer value is currently in the A register.
