use std::{
    fmt::Debug,
    mem::{size_of, MaybeUninit},
    ptr,
};

use crate::{linker::Instruction, linker::Instruction::*};

#[derive(Clone, Copy)]
union AticObj<'v> {
    as_number: f64,
    as_text: &'v String,
}

pub struct VM<'a> {
    instructions: Box<[Instruction]>,
    stack: Box<[AticObj<'a>; 65536]>,
    call_buffer: Box<[AticObj<'a>; 255]>,
    call_stack: Box<[i32; 65536]>,
    activation_record_pointer: i32,
    stack_pointer: i32,
    active_record_size: i32,
    pc: i32,
    running: bool,
    pub dbg_iter: i64,
    exit_code: f64,
}

impl VM<'_> {
    pub fn new(instructions: Vec<Instruction>) -> VM<'static> {
        let callbuffer = Box::new([AticObj { as_number: 0.0 }; 255]);
        let array = Box::new([AticObj { as_number: 0.0 }; 65536]);
        VM {
            instructions: instructions.into_boxed_slice(),
            stack: array,
            call_buffer: callbuffer,
            call_stack: Box::new([0; 65536]),
            activation_record_pointer: 0,
            stack_pointer: 0,
            active_record_size: 0,
            pc: 0,
            running: true,
            dbg_iter: 0,
            exit_code: 0.0,
        }
    }
    pub fn start(&mut self, adress: i32, arg_size: i32) {
        self.pc += 1;
        self.call_stack[0] = self.pc + 1;
        self.call_stack[1] = self.active_record_size;
        self.call_stack[2] = 0;
        self.activation_record_pointer += self.active_record_size;
        self.stack_pointer += 3;
        self.active_record_size += arg_size;
        self.pc = adress;
    }

    pub fn tick(&mut self) {
        // if !self.running {
        //     return;
        // }
        while self.running {
            self.dbg_iter += 1;
            let instruction = &self.instructions[self.pc as usize];

            match instruction {
                Debug(register) => {
                    let object = &self.stack[(self.activation_record_pointer + *register as i32) as usize];
                    unsafe {
                        println!("> {}: {}", register, object.as_number);
                    }
                    self.pc += 1;
                }
                Add(target, src_a, src_b) => {
                    let s1 = &self.stack[(self.activation_record_pointer + *src_a as i32) as usize];
                    let s2 = &self.stack[(self.activation_record_pointer + *src_b as i32) as usize];
                    unsafe {
                        self.stack[(self.activation_record_pointer + *target as i32) as usize] = AticObj {
                            as_number: s1.as_number + s2.as_number,
                        };
                    }
                    self.pc += 1;
                }
                Smaller(target, src_a, src_b) => {
                    let s1 = &self.stack[(self.activation_record_pointer + *src_a as i32) as usize];
                    let s2 = &self.stack[(self.activation_record_pointer + *src_b as i32) as usize];

                    unsafe {
                        self.stack[(self.activation_record_pointer + *target as i32) as usize] = AticObj {
                            as_number: if s1.as_number < s2.as_number {
                                1.0
                            } else {
                                0.0
                            },
                        };
                    }
                    self.pc += 1;
                }
                LoadConst(target, constant) => {
                    self.stack[(self.activation_record_pointer + *target as i32) as usize] = AticObj {
                        as_number: *constant,
                    };
                    println!(
                        "Loaded {} to {}",
                        *constant,
                        (self.activation_record_pointer + *target as i32)
                    );
                    self.pc += 1;
                }
                Jump(target) => {
                    self.pc = target.adress;
                }
                JumpIfNot(register, target) => {
                    let s1 = &self.stack[(self.activation_record_pointer + *register as i32) as usize];
                    unsafe {
                        if s1.as_number < 0.5 {
                            self.pc = target.adress;
                        } else {
                            self.pc += 1;
                        };
                    }
                }
                Exit(code) => {
                    self.running = false;
                    self.exit_code = *code;
                    println!("Exit with code {}", *code);
                }
                _ => {
                    panic!("Instruction not implemented");
                }
            }
        }
    }

    pub fn running(&self) -> bool {
        self.running
    }
    pub fn exit_code(&self) -> f64 {
        self.exit_code
    }
}

pub struct Callable {
    pub name: Box<String>,
    pub registers: i32,
    pub adress: i32,
    pub args: i32,
    pub capture_size: i32,
    pub capture: Box<[Callable]>,
}
