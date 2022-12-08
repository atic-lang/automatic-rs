use std::{collections::{HashMap, btree_map::Entry}, vec};

use crate::{ParseEntry::*, UnparsedInstruction};
use crate::{linker::Instruction::*};

extern crate fxhash;
use fxhash::FxHashMap;

use crate::{vm::Callable, ParseEntry};
#[derive(Default)]
pub struct Linker {
    instructions: Vec<Instruction>,
}    

impl Linker {
    fn push(&mut self, obj: Instruction) {
        self.instructions.push(obj);
    }

    pub fn feed_instructions(&mut self, instructions: &Vec<ParseEntry>) -> Vec<Callable> {
        let mut resolved = FxHashMap::<i32, i32>::default();
        let mut to_resolve = FxHashMap::<i32, &dyn Fn(i32)>::default();
        let additional_callables = vec![];
        for entry in instructions {
            let index = self.instructions.len();
            match &entry {
                ParseLabel(str) => {
                    let label = str.parse::<i32>().unwrap();
                    if let Some(consumer) = to_resolve.get(&label) {
                        consumer(label);
                        to_resolve.remove(&label);
                    }
                    resolved.insert(label, index as i32);
                },
                ParseInstruction(UnparsedInstruction {name, params}) => {
                    let empty = String::new();

                    let s1 = params.get(0).unwrap_or(&empty);
                    let s2 = params.get(1).unwrap_or(&empty);
                    let s3 = params.get(2).unwrap_or(&empty);

                    let i0 = s1.parse::<f64>().unwrap_or(0.0) as i32;
                    let i1 = s2.parse::<f64>().unwrap_or(0.0) as i32;
                    let i2 = s3.parse::<f64>().unwrap_or(0.0) as i32;

                    let f0 = s1.parse::<f64>().unwrap_or(0.0);
                    let f1 = s2.parse::<f64>().unwrap_or(0.0);
                    let f2 = s3.parse::<f64>().unwrap_or(0.0);

                    print!("Instruction {} ", name);
                    print!(" {} ", i0);
                    print!(" {} ", i1);
                    println!(" {} ", i2);
                    
                    self.push(match name.as_str() {
                        "Add" => {
                            Add(i0, i1, i2)
                        },
                        "LoadConst" => {
                            LoadConst(i0, f0)
                        },
                        "Debug" => {
                            Debug(i0)
                        },
                        _ => {
                            panic!("Unknown cmd {}", name)
                        }
                    });
                }
            }
        }

        additional_callables
    }
}

type Register = i32;
pub enum Instruction {
    Nop,
    Debug(Register),
    LoadConst(Register, f64),
    Copy(Register, Register),
    Not(Register, Register),
    Negate(Register, Register),
    LoadString(Register, String),
    LoadFunction(Register, Callable),
    Argument(i32, Register),
    Exit(f64),
    InvokeFunction(Register, Register),
    Return(Register),
    JumpIfNot(Register, Label),
    Jump(Label),
    LoadMember(Register, Register, i32),
    LoadArray(Register, Register, Register),
    StoreMember(Register, Register, i32),
    StoreArray(Register, Register, Register),
    CreateStruct(Register, i32),
    CreateEnumEntry(Register, i32, i32),
    CreateClosure(Register, i32),
    LoadEnumType(Register, Register),
    LoadEnumMember(Register, Register, i32),
    CopyEnumMember(Register, Register, i32),
    Throw(Register),
    Match(Register, Label, FxHashMap<Register, Label>),
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

struct Label {
    index: i32,
    adress: i32,
}
