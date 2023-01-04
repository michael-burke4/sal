use std::io;

fn main() -> io::Result<()> {
    for arg in std::env::args().skip(1) {
        let file_contents = std::fs::read_to_string(arg).unwrap();
        let lines = file_contents.lines();
        let lines_vec = lines.map(|s| s.to_string()).collect();
        let mut eval = Evaluator::new(lines_vec);
        eval.run();
        eval.print();
    }
    Ok(())
}

#[derive (Clone, Debug, PartialEq)]
enum Val {
    Str(String),
    Int(i64),
    Float(f64),
}

impl Val {

    // Value add function with built in type coercion. Any addition of values will be valid,
    // but types will be changed as necessary.
    // string + {any type} -> string
    // float + float -> float
    // float + int -> float
    // int + int -> int
    pub fn add(self, other: &Val) -> Self {
        match self {
            Val::Str(string) => {
                match other {
                    Val::Str(string2) => {
                        return Val::Str(format!("{}{}", string, string2));
                    },
                    Val::Float(flt2) => {
                        return Val::Str(format!("{}{}", string, flt2));
                    },
                    Val::Int(int2) => {
                        return Val::Str(format!("{}{}", string, int2));
                    }
                }
            },
            Val::Float(flt) => {
                match other {
                    Val::Str(string2) => {
                        return Val::Str(format!("{}{}", flt, string2));
                    },
                    Val::Float(flt2) => {
                        return Val::Float(flt + flt2);
                    },
                    Val::Int(int2) => {
                        return Val::Float(flt + (*int2 as f64));
                    }
                }
            },
            Val::Int(int) => {
                match other {
                    Val::Str(string2) => {
                        return Val::Str(format!("{}{}", int, string2));
                    },
                    Val::Float(flt2) => {
                        return Val::Float((int as f64) + flt2);
                    },
                    Val::Int(int2) => {
                        return Val::Int(int + int2);
                    }
                }
            }
        }
    }
}

fn parse_value(string: &str) -> Option<Val> {
    if string.starts_with("\"") && string.ends_with("\"") {
        return Some(Val::Str((string[1..string.len()-1]).to_string()));
    }
    if string.contains(".") {
        let val = string.parse::<f64>();
        match val {
            Ok(fvalue) => return Some(Val::Float(fvalue)),
            _ => return None,
        }
    }

    let val = string.parse::<i64>();
    match val {
        Ok(ivalue) => return Some(Val::Int(ivalue)),
        _ => return None,
    }
}

#[derive (Clone, Copy, Debug, PartialEq)]
enum EvaluatorErr{
    LineOutOfBounds(usize),
    ArgMismatch(usize, usize, usize),
    ParseValueError(usize),
    EmptyStack(usize),
    HaltedStep,
    UnsupportedOperation(usize),
}
struct Evaluator{ 
    stack: Vec<Val>,
    // program: Vec<Result<String, std::io::Error>>,
    program: Vec<String>,
    program_counter: usize,
    register_a: Val,
    register_b: Val,
    register_r: usize,
    halt: bool,
}

impl Evaluator {
    pub fn new(program_vector: Vec<String>) -> Self {
       Evaluator {
            stack: vec![],
            program: program_vector,
            program_counter: 0,
            register_a: Val::Int(0),
            register_b: Val::Int(0),
            register_r: 0,
            halt: false,
       } 
    }
    pub fn print(&self) {
        println!("A:{:?} B:{:?} R:{} PC:{}", self.register_a, self.register_b, self.register_r, self.program_counter+1);
        println!("Stack:");
        self.print_stack();
    }

    fn pushi(&mut self, value: Val) -> Result<(), EvaluatorErr> {
        self.stack.push(value);
        self.program_counter += 1;
        Ok(())
    }
    
    fn pushr(&mut self) -> Result<(), EvaluatorErr> {
        self.stack.push(self.register_a.clone());
        self.program_counter += 1;
        return Ok(());
    }
    
    pub fn pops(&mut self) -> Result<(), EvaluatorErr> {
        match self.stack.pop() {
            Some(value) => {
                self.program_counter += 1;
                self.register_a = value;
                return Ok(());
            },
            None => return Err(EvaluatorErr::EmptyStack(self.program_counter)),
        }
    }

    pub fn pop(&mut self) -> Result<(), EvaluatorErr> {
        match self.stack.pop() {
            Some(_) => {
                self.program_counter += 1;
                return Ok(());
            },
            None => return Err(EvaluatorErr::EmptyStack(self.program_counter)),
        }
    }    

    fn peek(&mut self) -> Result<(), EvaluatorErr> {
        if self.stack.len() == 0 {
            return Err(EvaluatorErr::EmptyStack(self.program_counter))
        }
        self.program_counter += 1;
        self.register_a = self.stack.get(self.stack.len() - 1).unwrap().clone();
        Ok(())
    }
    
    fn jump(&mut self, line: usize) -> Result<(), EvaluatorErr> {
        self.program_counter = line - 1;
        Ok(())
    }
    
    fn jzer(&mut self, line: usize) -> Result<(), EvaluatorErr> {
        if self.register_a == Val::Int(0) {
            self.program_counter = line-1;
        }
        else {
            self.program_counter += 1;
        }
        Ok(())
    }
    
    fn call(&mut self, line: usize) -> Result<(), EvaluatorErr> {
        self.register_r = self.program_counter;
        self.program_counter = line - 1;
        Ok(())
    }

    fn ret(&mut self) -> Result<(), EvaluatorErr> {
        self.program_counter = self.register_r;
        Ok(())
    }
    
    fn add(&mut self) -> Result<(), EvaluatorErr> {
        if self.stack.len() < 2 {
            return Err(EvaluatorErr::EmptyStack(self.program_counter))
        }
        self.program_counter += 1;
        let stack_top_index = self.stack.len()-1;
        self.register_a = self.stack.get(stack_top_index).unwrap().clone().add(self.stack.get(stack_top_index-1).unwrap());
        Ok(())
    }
    
    // Follows similar type rules to add, but raises an error when trying to mult with any string.
    fn mult(&mut self) -> Result<(), EvaluatorErr> {
        if self.stack.len() < 2 {
            return Err(EvaluatorErr::EmptyStack(self.program_counter))
        }

        let stack_top_index = self.stack.len()-1;
        let lhs = self.stack.get(stack_top_index).unwrap().clone();
        let rhs = self.stack.get(stack_top_index-1).unwrap().clone();
        match lhs {
            Val::Str(_) => {
                return Err(EvaluatorErr::UnsupportedOperation(self.program_counter));
            },
            Val::Float(flt1) => {
                match rhs {
                    Val::Str(_) => {
                        return Err(EvaluatorErr::UnsupportedOperation(self.program_counter));
                    },
                    Val::Float(flt2) => {
                        self.register_a = Val::Float(flt1*flt2);
                    },
                    Val::Int(int2) => {
                        self.register_a = Val::Float(flt1 * (int2 as f64));
                    }
                }
            },
            Val::Int(int1) => {
                match rhs {
                    Val::Str(_) => {
                        return Err(EvaluatorErr::UnsupportedOperation(self.program_counter));
                    },
                    Val::Float(flt2) => {
                        self.register_a = Val::Float((int1 as f64) *flt2);
                    },
                    Val::Int(int2) => {
                        self.register_a = Val::Int(int1 * int2);
                    }
                }
            },
        }

        self.program_counter += 1;    
        Ok(())
    }

    fn swap(&mut self) -> Result<(), EvaluatorErr> {
        let temp = self.register_a.clone();
        self.register_a = self.register_b.clone();
        self.register_b = temp; 
        self.program_counter += 1;
        Ok(())
    }
    
    fn inc(&mut self) -> Result<(), EvaluatorErr> {
        match self.register_a {
            Val::Int(int) => {
                self.register_a = Val::Int(int+1);
                self.program_counter += 1;
                Ok(())
            },
            _ => Err(EvaluatorErr::UnsupportedOperation(self.program_counter))
        }
    }

    fn dec(&mut self) -> Result<(), EvaluatorErr> {
        match self.register_a {
            Val::Int(int) => {
                self.register_a = Val::Int(int-1);
                self.program_counter += 1;
                Ok(())
            }
            _ => Err(EvaluatorErr::UnsupportedOperation(self.program_counter))
        }
    }
    pub fn print_stack(&self) {
        for element in &self.stack {
            println!("{:?}", element);
        }
    }
    
    pub fn run(&mut self) {
        while !self.halt {
            let step_result = self.step();
            match step_result {
                Ok(()) => continue,
                Err(error) => {
                    match error {
                        EvaluatorErr::LineOutOfBounds(line) => {
                            let line = line+1;
                            panic!("Line {} out of bounds", line);
                        },
                        EvaluatorErr::ArgMismatch(line, expected, got) => {
                            let line = line+1;
                           panic!("Argument error on line {line}: expected {expected}, got {got}");
                        },
                        EvaluatorErr::ParseValueError(line) => {
                            let line = line+1;
                            panic!("Parse error, could not parse supplied value on line {line}");
                        },
                        EvaluatorErr::EmptyStack(line) => {
                            let line = line+1;
                            panic!("Stack error, tried to use out of bounds stack index on line {line}");
                        },
                        EvaluatorErr::HaltedStep => {
                            panic!("Tried to step evaluator after execution halted");
                        },
                        EvaluatorErr::UnsupportedOperation(line) => {
                            let line = line+1;
                            panic!("Tried to perform an unsupported operation on line {line}. Check stack and make sure you're not multiplying/dividing/subtracting using strings!");
                        }
                    }
                }
            }
        }
    }

    pub fn step(&mut self) -> Result<(), EvaluatorErr> {
        if self.halt {return Err(EvaluatorErr::HaltedStep)}
        let next_command = self.program.get(self.program_counter);
        match next_command {
            Some(line) => {
                let tokens: Vec<_> = line.split_ascii_whitespace().collect();
                match tokens.get(0) {
                    Some(string) => {
                        match *string {
                            "pushi" => {
                            if tokens.len() != 2 {return Err(EvaluatorErr::ArgMismatch(self.program_counter, tokens.len()-1, 1))}
                                let value = parse_value(*(tokens.get(1).unwrap()));
                                match value {
                                    Some(something) => return self.pushi(something),
                                    None => return Err(EvaluatorErr::ParseValueError(self.program_counter)),
                                }
                            },
                            "pushr" => {
                                if tokens.len() != 1 {return Err(EvaluatorErr::ArgMismatch(self.program_counter, tokens.len()-1, 0))}
                                return self.pushr();
                            },
                            "inc" => {
                                if tokens.len() != 1 {return Err(EvaluatorErr::ArgMismatch(self.program_counter, tokens.len()-1, 0))}
                                return self.inc()
                            },
                            "dec" => {
                                if tokens.len() != 1 {return Err(EvaluatorErr::ArgMismatch(self.program_counter, tokens.len()-1, 0))}
                                return self.dec()
                            },
                            "pop" => {
                                if tokens.len() != 1 {return Err(EvaluatorErr::ArgMismatch(self.program_counter, tokens.len()-1, 0))}
                                return self.pop()
                            },
                            "pops" => {
                                if tokens.len() != 1 {return Err(EvaluatorErr::ArgMismatch(self.program_counter, tokens.len()-1, 0))}
                                return self.pops()
                            },
                            "peek" => {
                                if tokens.len() != 1 {return Err(EvaluatorErr::ArgMismatch(self.program_counter, tokens.len()-1, 0))}
                                return self.peek()
                            },
                            "swap" => {
                                if tokens.len() != 1 {return Err(EvaluatorErr::ArgMismatch(self.program_counter, tokens.len()-1, 0))}
                                return self.swap()
                            },
                            "add" => {
                                if tokens.len() != 1 {return Err(EvaluatorErr::ArgMismatch(self.program_counter, tokens.len()-1, 0))}
                                return self.add() 
                            },
                            "mult" => {
                                if tokens.len() != 1 {return Err(EvaluatorErr::ArgMismatch(self.program_counter, tokens.len()-1, 0))}
                                return self.mult() 
                            },
                            "div" => {
                                println!("DIV!");
                            },
                            "jump" => {
                                if tokens.len() != 2 {return Err(EvaluatorErr::ArgMismatch(self.program_counter, tokens.len()-1, 1))}
                                let jump_loc = tokens[1].parse::<usize>();
                                // let value = parse_value(*(tokens.get(1).unwrap()));
                                match jump_loc {
                                    Ok(addr) => return self.jump(addr),
                                    Err(_) => return Err(EvaluatorErr::ParseValueError(self.program_counter)),
                                }
                            },
                            "jzer" => {
                                if tokens.len() != 2 {return Err(EvaluatorErr::ArgMismatch(self.program_counter, tokens.len()-1, 1))}
                                let jump_loc = tokens[1].parse::<usize>();
                                // let value = parse_value(*(tokens.get(1).unwrap()));
                                match jump_loc {
                                    Ok(addr) => return self.jzer(addr),
                                    Err(_) => return Err(EvaluatorErr::ParseValueError(self.program_counter)),
                                }
                            },
                            "call" => {
                                if tokens.len() != 2 {return Err(EvaluatorErr::ArgMismatch(self.program_counter, tokens.len()-1, 1))}
                                let jump_loc = tokens[1].parse::<usize>();
                                // let value = parse_value(*(tokens.get(1).unwrap()));
                                match jump_loc {
                                    Ok(addr) => return self.call(addr),
                                    Err(_) => return Err(EvaluatorErr::ParseValueError(self.program_counter)),
                                }
                            },
                            "ret" => {
                                if tokens.len() != 1 {return Err(EvaluatorErr::ArgMismatch(self.program_counter, tokens.len()-1, 0))}
                                return self.ret()
                            },
                            "halt" => {
                                self.halt = true;
                                return Ok(());
                            },
                            _ => {
                                println!("UNRECOGNIZED!!");
                                return Err(EvaluatorErr::ParseValueError(self.program_counter));
                            }
                        }
                        return Ok(())
                    },
                    None => {
                        //this is the blank line case
                        self.program_counter += 1;
                        return Ok(())
                    },
                }
            },
            None => Err(EvaluatorErr::LineOutOfBounds(self.program_counter)),
        }
    }
}

#[test]
fn val_add_test() {
    let val1 = Val::Str("Hello".to_string());
    let val2 = Val::Float(3.14);
    let val3 = Val::Int(5);
    assert_eq!(val1.clone().add(&val2), Val::Str("Hello3.14".to_string()));
    assert_eq!(val1.clone().add(&val3), Val::Str("Hello5".to_string()));
    assert_eq!(val2.clone().add(&val3), Val::Float(8.14));
    assert_eq!(val2.clone().add(&val2), Val::Float(6.28));
    assert_eq!(val3.clone().add(&val3), Val::Int(10));
}

