use std::fs::File;
use std::io::{self, prelude::*, BufReader};

fn main() -> io::Result<()> {
    let file = File::open("../main.txt")?;
    let reader = BufReader::new(file);
    let lines_vec: Vec<_> = reader.lines().collect();
    let mut eval = Evaluator::new(lines_vec);
    let _ = eval.step();
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
        return Some(Val::Str(string.to_string()));
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
    // BadLineError(usize),
    ArgMismatch(usize, usize, usize),
    ParseValueError(usize),
    EmptyStack(usize),
    HaltedStep,
}
struct Evaluator {
    stack: Vec<Val>,
    program: Vec<Result<String, std::io::Error>>,
    program_counter: usize,
    registerA: Val,
    registerB: Val,
    registerR: usize,
    halt: bool,
}

impl Evaluator {
    pub fn new(program_vector: Vec<Result<String, std::io::Error>>) -> Self {
       Evaluator {
            stack: vec![],
            program: program_vector,
            program_counter: 0,
            registerA: Val::Int(0),
            registerB: Val::Int(0),
            registerR: 0,
            halt: false,
       } 
    }

    fn pushi(&mut self, value: Val) -> Result<(), EvaluatorErr> {
        self.stack.push(value);
        self.program_counter += 1;
        Ok(())
    }
    
    fn pushr(&mut self) -> Result<(), EvaluatorErr> {
        self.stack.push(self.registerA.clone());
        return Ok(());
    }
    
    pub fn pops(&mut self) -> Result<(), EvaluatorErr> {
        match self.stack.pop() {
            Some(value) => {
                self.program_counter += 1;
                self.registerA = value;
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
        self.registerA = self.stack.get(self.stack.len() - 1).unwrap().clone();
        Ok(())
    }
    
    fn jump(&mut self, line: usize) -> Result<(), EvaluatorErr> {
        self.program_counter = line - 1;
        Ok(())
    }
    
    fn call(&mut self, line: usize) -> Result<(), EvaluatorErr> {
        self.registerR = self.program_counter;
        self.program_counter = line - 1;
        Ok(())
    }

    fn ret(&mut self) -> Result<(), EvaluatorErr> {
        self.program_counter = self.registerR;
        Ok(())
    }
    
    fn add(&mut self) -> Result<(), EvaluatorErr> {
        if self.stack.len() < 2 {
            return Err(EvaluatorErr::EmptyStack(self.program_counter))
        }
        let stack_top_index = self.stack.len()-1;
        self.registerA = self.stack.get(stack_top_index).unwrap().clone().add(self.stack.get(stack_top_index-1).unwrap());
        Ok(())
    }
    
    fn swap(&mut self) -> Result<(), EvaluatorErr> {
       let temp = self.registerA.clone();
       self.registerA = self.registerB.clone();
       self.registerB = temp; 
       Ok(())
    }

    pub fn print_stack(&self) {
        for element in &self.stack {
            println!("{:?}", element);
        }
    }

    pub fn step(&mut self) -> Result<(), EvaluatorErr> {
        if self.halt {return Err(EvaluatorErr::HaltedStep)}
        let next_command = self.program.get(self.program_counter);
        match next_command {
            Some(str_result) => {
                match str_result {
                    Ok(line) => {
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
                                    }
                                    "add" => {
                                        if tokens.len() != 1 {return Err(EvaluatorErr::ArgMismatch(self.program_counter, tokens.len()-1, 0))}
                                        return self.add() 
                                    },
                                    "mult" => {
                                        println!("MULT!");
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
                                            Ok(addr) => return self.jump(addr),
                                            Err(_) => return Err(EvaluatorErr::ParseValueError(self.program_counter)),
                                        }
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
                            None => return Ok(()),
                        }
                    },
                    Err(_) => return Err(EvaluatorErr::LineOutOfBounds(self.program_counter)),
                }
            },
            None => return Err(EvaluatorErr::LineOutOfBounds(self.program_counter)),
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

