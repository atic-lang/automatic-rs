use std::{mem::MaybeUninit, ptr};

use crate::{linker::Instruction};

#[derive(Clone, Copy)]
union AticObj<'v> {
    Number: f64,
    Text: &'v String,
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
    dbg_iter: i64,
}

impl VM<'_> {
    pub fn new(instructions: Vec<Instruction>) -> VM<'static> {
        let callbuffer = Box::new([AticObj {Number: 0.0}; 255]);
        let array = Box::new([AticObj {Number: 0.0}; 65536]);
        VM {
            instructions: instructions.into_boxed_slice(),
            stack: array,
            call_buffer: callbuffer,
            call_stack: Box::new([0; 65536]) ,
            activation_record_pointer: 0,
            stack_pointer: 0,
            active_record_size: 0,
            pc: 0,
            running: true,
            dbg_iter: 0,
        }
    }
    pub fn start(&mut self, string: &str) {
        let function_input = 0;
        let adress = 0;
        self.pc += 1;
        self.call_stack[0] = self.pc + 1;
        self.call_stack[1] = self.active_record_size;
        self.call_stack[2] = 0;
        self.activation_record_pointer += self.active_record_size;
        self.stack_pointer += 3;
        self.active_record_size += function_input;
        self.pc = adress;
    }
}

pub struct Callable {
    registers: i32,
    adress: i32,
    args: i32,
    capture_size: i32,
    capture: Box<[Callable]>,
}
    

