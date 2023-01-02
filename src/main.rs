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
}
struct Evaluator {
    stack: Vec<Val>,
    program: Vec<Result<String, std::io::Error>>,
    program_counter: usize,
    register: Val,
}

impl Evaluator {
    pub fn new(program_vector: Vec<Result<String, std::io::Error>>) -> Self {
       Evaluator {
            stack: vec![],
            program: program_vector,
            program_counter: 0,
            register: Val::Int(0),
       } 
    }

    fn push(&mut self, value: Val) -> Result<(), EvaluatorErr>{
        self.stack.push(value);
        self.program_counter += 1;
        Ok(())
    }
    
    pub fn pops(&mut self) -> Result<(), EvaluatorErr> {
        match self.stack.pop() {
            Some(value) => {
                self.program_counter += 1;
                self.register = value;
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
        self.register = self.stack.get(self.stack.len() - 1).unwrap().clone();
        Ok(())
    }
    
    fn jump(&mut self, line: usize) -> Result<(), EvaluatorErr> {
        self.program_counter = line - 1;
        return Ok(());
    }

    pub fn print_stack(&self) {
        for element in &self.stack {
            println!("{:?}", element);
        }
    }

    pub fn step(&mut self) -> Result<(), EvaluatorErr>{
        let next_command = self.program.get(self.program_counter);
        match next_command {
            Some(str_result) => {
                match str_result {
                    Ok(line) => {
                        let tokens: Vec<_> = line.split_ascii_whitespace().collect();
                        match tokens.get(0) {
                            Some(string) => {
                                match *string {
                                    "push" => {
                                        if tokens.len() != 2 {return Err(EvaluatorErr::ArgMismatch(self.program_counter, tokens.len()-1, 1))}
                                        let value = parse_value(*(tokens.get(1).unwrap()));
                                        match value {
                                            Some(something) => return self.push(something),
                                            None => return Err(EvaluatorErr::ParseValueError(self.program_counter)),
                                        }
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
                                        println!("ADD!");
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
                                        println!("JZER!");
                                    },
                                    _ => println!("UNRECOGNIZED!!")
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