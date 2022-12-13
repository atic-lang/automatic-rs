use std::{
    collections::{btree_map::Entry, HashMap},
    vec,
};

use crate::linker::Instruction::*;
use crate::{ParseEntry::*, UnparsedInstruction};

extern crate fxhash;
use fxhash::FxHashMap;

use crate::{vm::Callable, ParseEntry};
#[derive(Default)]
pub struct Linker {
    pub instructions: Vec<Instruction>,
}

impl Linker {
    fn push(&mut self, obj: Instruction) {
        self.instructions.push(obj);
    }

    pub fn feed_instructions(&mut self, instructions: &Vec<ParseEntry>) -> Vec<Callable> {
        let mut labels = FxHashMap::<i32, i32>::default();
        let additional_callables = vec![];
        let mut function_instructions = vec![];
        for entry in instructions {
            let index = (function_instructions.len() + self.instructions.len()) as i32;
            match &entry {
                ParseLabel(str) => {
                    let label = str.parse::<f64>().unwrap() as i32;
                    labels.insert(label, index);
                }
                ParseInstruction(UnparsedInstruction { name, params }) => {
                    let empty = String::new();

                    let s1 = params.get(0).unwrap_or(&empty);
                    let s2 = params.get(1).unwrap_or(&empty);
                    let s3 = params.get(2).unwrap_or(&empty);

                    let i0 = s1.parse::<f64>().unwrap_or(0.0) as i8;
                    let i1 = s2.parse::<f64>().unwrap_or(0.0) as i8;
                    let i2 = s3.parse::<f64>().unwrap_or(0.0) as i8;

                    let f0 = s1.parse::<f64>().unwrap_or(0.0);
                    let f1 = s2.parse::<f64>().unwrap_or(0.0);
                    let f2 = s3.parse::<f64>().unwrap_or(0.0);

                    print!("Instruction {} ", name);
                    print!(" {} ", i0);
                    print!(" {} ", i1);
                    println!(" {} ", i2);

                    function_instructions.push(match name.as_str() {
                        "Add" => Add(i0, i1, i2),
                        "Smaller" => Smaller(i0, i1, i2),
                        "LoadConst" => LoadConst(i0, f1),
                        "Exit" => Exit(f0),
                        "Debug" => Debug(i0),
                        "Argument" => Argument(i0, i1),
                        "CreateStruct" => CreateStruct(i0, i1),
                        "Jump" => Jump(Box::new(Label {
                            index: i0,
                            adress: -1,
                        })),
                        "JumpIfNot" => JumpIfNot(
                            i0,
                            Box::new(Label {
                                index: i1,
                                adress: -1,
                            }),
                        ),
                        _ => {
                            panic!("Unknown cmd {}", name)
                        }
                    });
                }
            }
        }
        for ele in function_instructions {
            self.push(match ele {
                Jump(target) => {
                    if let Some(adress) = labels.get(&(target.index as i32)) {
                        Jump(Box::new(Label {
                            index: target.index,
                            adress: *adress,
                        }))
                    } else {
                        panic!("Jump: Cant find Adress of Label {}", target.index);
                    }
                }
                JumpIfNot(register, target) => {
                    if let Some(adress) = labels.get(&(target.index as i32)) {
                        JumpIfNot(
                            register,
                            Box::new(Label {
                                index: target.index,
                                adress: *adress,
                            }),
                        )
                    } else {
                        panic!("JumpIfNot: Cant find Adress of Label {}", target.index);
                    }
                }
                rest => rest,
            });
        }

        additional_callables
    }
}

type Register = i8;
type Offset = i8;
pub enum Instruction {
    Nop,
    Debug(Register),
    LoadConst(Register, f64),
    Copy(Register, Register),
    Not(Register, Register),
    Negate(Register, Register),
    LoadString(Register, Box<String>),
    LoadFunction(Register, Box<Callable>),
    Argument(Offset, Register),
    Exit(f64),
    InvokeFunction(Register, Register),
    Return(Register),
    JumpIfNot(Register, Box<Label>),
    Jump(Box<Label>),
    LoadMember(Register, Register, Offset),
    LoadArray(Register, Register, Register),
    StoreMember(Register, Register, Offset),
    StoreArray(Register, Register, Register),
    CreateStruct(Register, Offset),
    CreateEnumEntry(Register, Offset, Offset),
    CreateClosure(Register, Offset),
    LoadEnumType(Register, Register),
    LoadEnumMember(Register, Register, Offset),
    CopyEnumMember(Register, Register, Offset),
    Throw(Register),
    Match(Register, Box<Label>, Box<FxHashMap<Register, Label>>),
    Add(Register, Register, Register),
    Subtract(Register, Register, Register),
    Multiply(Register, Register, Register),
    Divide(Register, Register, Register),
    Or(Register, Register, Register),
    And(Register, Register, Register),
    Greater(Register, Register, Register),
    GreaterEq(Register, Register, Register),
    Smaller(Register, Register, Register),
    SmallerEq(Register, Register, Register),
    Equals(Register, Register, Register),
    NonEquals(Register, Register, Register),
    StringNonEquals(Register, Register, Register),
    StringEquals(Register, Register, Register),
    Concat(Register, Register, Register),
}

pub struct Label {
    pub index: i8,
    pub adress: i32,
}
