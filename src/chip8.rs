use rand;

use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;

pub struct Chip8 {

    
    opcode: u16,
    
    memory: [u8; 4096],

    registers: [u8; 16],

    indexing: u16,

    program_counter: usize,

    gfx: [u8; 64*32],

    delay_timer: u8,

    sound_timer: u8,

    stack: [u16; 16],

    stack_pointer: usize,

    keys: [u8; 16],

    jump_table: HashMap<u16, fn(&mut Chip8) >,

    wait_index: usize,
}


impl Chip8{

    
    
    pub fn new(path: &str) -> Chip8{
       let mut chip8 = Chip8{
            opcode:0,
            memory: [0;4096],
            registers: [0;16],
            indexing:0,
            program_counter:0x200,
            gfx: [0;64*32],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0;16],
            stack_pointer: 0,
            keys: [0;16],
            jump_table: HashMap::new(),
            wait_index: 255,
        };


        chip8.load_game(path);
       
        chip8.load_fontset();

        chip8.init_jumptable();

        chip8
    }

   
    fn load_fontset(&mut self){
        let fontset = vec!(0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
                           0x20, 0x60, 0x20, 0x20, 0x70, // 1
                           0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
                           0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
                           0x90, 0x90, 0xF0, 0x10, 0x10, // 4
                           0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
                           0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
                           0xF0, 0x10, 0x20, 0x40, 0x40, // 7
                           0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
                           0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
                           0xF0, 0x90, 0xF0, 0x90, 0x90, // A
                           0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
                           0xF0, 0x80, 0x80, 0x80, 0xF0, // C
                           0xE0, 0x90, 0x90, 0x90, 0xE0, // D
                           0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
                           0xF0, 0x80, 0xF0, 0x80, 0x80  // F
                           );

        let end = (0x50 + fontset.len()) as usize;
        for i in 0x50..end {
            self.memory[i] = fontset[i-0x50];
        }

    }

    pub fn load_game(&mut self, path: &str){
        let file = File::open(path).unwrap();
        

        let mut i= 512;

        let content: Vec<Result<u8,io::Error>> = file.bytes().collect();

        println!("filesize: {}", content.len());

        for b  in  content{
            self.memory[i] = b.unwrap();
            i = i + 1;
        }
    }

    fn init_jumptable(&mut self){

        self.jump_table.insert(0x00E0, Self::op_cls );
        self.jump_table.insert(0x00EE, Self::op_ret );
        self.jump_table.insert(0x1000, Self::op_jmp );
        self.jump_table.insert(0x2000, Self::op_call );
        self.jump_table.insert(0x3000, Self::op_se_vx_byte);
        self.jump_table.insert(0x4000, Self::op_sne_vx_byte);
        self.jump_table.insert(0x5000, Self::op_se_vx_vy);
        self.jump_table.insert(0x6000, Self::op_ld_vx_byte);
        self.jump_table.insert(0x7000, Self::op_add_vx_byte);

        self.jump_table.insert(0x8000, Self::op_ld_vx_vy);
        self.jump_table.insert(0x8001, Self::op_or_vx_vy);
        self.jump_table.insert(0x8002, Self::op_and_vx_vy);
        self.jump_table.insert(0x8003, Self::op_xor_vx_vy);
        self.jump_table.insert(0x8004, Self::op_add_vx_vy);
        self.jump_table.insert(0x8005, Self::op_sub_vx_vy);
        self.jump_table.insert(0x8006, Self::op_shr_vx_vy);
        self.jump_table.insert(0x8007, Self::op_subn_vx_vy);
        self.jump_table.insert(0x800E, Self::op_shl_vx_vy);

        self.jump_table.insert(0x9000, Self::op_sne_vx_vy);
        self.jump_table.insert(0xA000, Self::op_ld_i);
        self.jump_table.insert(0xB000, Self::op_jmp_v0);
        self.jump_table.insert(0xC000, Self::op_rnd);
        self.jump_table.insert(0xD000, Self::op_drw);
        
        self.jump_table.insert(0xE09E, Self::op_skp);
        self.jump_table.insert(0xE0A1, Self::op_sknp);
        
        self.jump_table.insert(0xF007, Self::op_ld_vx_dt);
        self.jump_table.insert(0xF00A, Self::op_ld_vx_k);
        self.jump_table.insert(0xF015, Self::op_ld_dt_vx);
        self.jump_table.insert(0xF018, Self::op_ld_st_vx);
        self.jump_table.insert(0xF01E, Self::op_add_i_vx);
        self.jump_table.insert(0xF029, Self::op_ld_f_vx);
        self.jump_table.insert(0xF033, Self::op_ld_b_vx);
        self.jump_table.insert(0xF055, Self::op_ld_i_vx);
        self.jump_table.insert(0xF065, Self::op_ld_vx_i);
    }

    pub fn emulate_cycle(&mut self){


        self.fetch();

      //  self.log_state();
        
        self.decode_and_execute();


       }

    /*
    fn debug_halt(&self){
         let mut input = String::new();
        io::stdin().read_line(&mut input);

    }

     fn log_state(&self){
        println!("--------AFTER FETCH -------\n");
        println!("opcode:{:x}  \npc at:{} I at:{} sp at:{} dt at:{}",
                                                           self.opcode,
                                                           self.program_counter,
                                                           self.indexing,
                                                           self.stack_pointer,
                                                           self.delay_timer);
        println!("registers:");
        for i in 0..self.registers.len(){
            print!( "v{:X}={}  ",i, self.registers[i]);    
        }
        print!("\n");

        println!("\nstack:");
        for i in 0..self.stack.len(){
            print!( "s{:X}={}  ",i, self.stack[i]);
        }
        println!("\n--------------------");
    }
    */

    pub fn decrease_dt(&mut self){
        if self.delay_timer == 0 {
           return
        }
        self.delay_timer = self.delay_timer  - 1;
    }

    fn fetch(&mut self){
        let upper = (self.memory[self.program_counter] as u16) << 8;
        let lower = self.memory[self.program_counter+1] as u16;

        self.opcode = upper | lower;


    }

    fn decode_and_execute(&mut self){
        let upper = self.opcode & 0xF000;

        if ( upper >= 0x1000 && upper < 0x8000 ) 
            ||  (upper == 0x9000)
            ||  (upper >= 0xA000 && upper < 0xE000) {
            self.execute_decoded(upper); 
        } 
        else if upper >= 0x8000 && upper <0x9000 {
            let key = self.opcode & 0xF00F;
            self.execute_decoded(key);
        }
        else if upper >= 0xE000 {
            let key = self.opcode & 0xF0FF;
            self.execute_decoded(key);
        }
        else {
            let key = self.opcode;
            self.execute_decoded(key);
        }

    }

    fn execute_decoded(&mut self, key: u16){
            let func = self.jump_table.get(&key).map(|x| *x);
            
            match func{
                Some(func) => func(self),
                None => {
                        println!("invalid op: {}", self.opcode);
                        sleep(Duration::from_millis(1000000));
                        return;
                }
            }();


            self.program_counter = self.program_counter + 2;
    }
    
    fn op_cls(&mut self){        
        self.gfx = [0; 64*32]
    }

    fn op_ret(&mut self){        
        self.program_counter = (self.stack[self.stack_pointer]) as usize;
        self.stack_pointer = self.stack_pointer -1;

    }
    
    fn op_jmp(&mut self){
        self.program_counter = ((self.opcode & 0x0FFF) -2) as usize;
    }
    
    fn op_call(&mut self){
        self.stack_pointer = self.stack_pointer + 1;
        self.stack[self.stack_pointer] = self.program_counter as u16;
        self.program_counter = ((self.opcode & 0x0FFF) - 2) as usize;
    }
   
    fn op_se_vx_byte(&mut self){
        let index = self.get_vx();
        let cmp = self.get_byte();

        if self.registers[index] == cmp{
            self.program_counter = self.program_counter + 2;
        }
    }
    
    fn op_sne_vx_byte(&mut self){
        let index = self.get_vx();       
        let cmp = self.get_byte();

        if self.registers[index] != cmp{
            self.program_counter = self.program_counter + 2;
        }
    }
   
    fn op_se_vx_vy(&mut self){
        let index_vx = self.get_vx();
        let index_vy = self.get_vy();

        if self.registers[index_vx] == self.registers[index_vy]{
            self.program_counter = self.program_counter + 2;
        }

    }
    
    fn op_ld_vx_byte(&mut self){
        let c = (self.opcode & 0x00FF) as u8;
        let index = self.get_vx();

        self.registers[index] = c;
    }
  
    fn op_add_vx_byte(&mut self){
        let byte = self.get_byte();
        let index = self.get_vx();

        let (sum, _) = self.add_with_carry(self.registers[index],byte);
        self.registers[index] = sum;

    }
         
    fn op_ld_vx_vy(&mut self){
        let index_vy = self.get_vy();
        let index_vx = self.get_vx();

        self.registers[index_vx] = self.registers[index_vy];
    }

    fn op_or_vx_vy(&mut self){
        let index_vy = self.get_vy();
        let index_vx = self.get_vx();

        self.registers[index_vx] = self.registers[index_vx] | self.registers[index_vy];
   }

    fn op_and_vx_vy(&mut self){
        let index_vy = self.get_vy();
        let index_vx = self.get_vx();

        self.registers[index_vx] = self.registers[index_vx] & self.registers[index_vy]; 
    }

    fn op_xor_vx_vy(&mut self){
        let index_vy = self.get_vy();
        let index_vx = self.get_vx();

        self.registers[index_vx] = self.registers[index_vx] ^ self.registers[index_vy]; 
   }

    fn op_add_vx_vy(&mut self){
        let index_vx = self.get_vx();
        let index_vy = self.get_vy();

        let result: u16 = self.registers[index_vx] as u16 + self.registers[index_vy] as u16;
        let carry = if result >  0  {0} else { 1 };

        self.registers[index_vx] = (result & 0x00FF) as u8;
        self.registers[15] = carry;
        
    }

    fn op_sub_vx_vy(&mut self){
        let index_vx = self.get_vx();
        let index_vy = self.get_vy();

        let result: i16 = self.registers[index_vx] as i16 - self.registers[index_vy] as i16;
        let carry = if self.registers[index_vx] > self.registers[index_vy] { 1 } else {0};

        self.registers[index_vx] = (result & 0x00FF) as u8;
        self.registers[15] = carry;


    }

    fn op_shr_vx_vy(&mut self){
        let index_vx = self.get_vx();

        let carry = self.registers[index_vx] & 0x01;

        self.registers[index_vx] = self.registers[index_vx] >> 1;
        self.registers[15] = carry;
    }

    fn op_subn_vx_vy(&mut self){
        let index_vx = self.get_vx();
        let index_vy = self.get_vy();

        let result: i16 = self.registers[index_vy] as i16 - self.registers[index_vx] as i16;
        let carry = if self.registers[index_vy] > self.registers[index_vx] { 1 } else {0};

        self.registers[index_vx] = (result & 0x00FF) as u8;
        self.registers[15] = carry;

   }

    fn op_shl_vx_vy(&mut self){
        let index_vx = self.get_vx();

        let carry = self.registers[index_vx] & 0x80;

        self.registers[index_vx] = self.registers[index_vx] << 1;
        self.registers[15] = carry;  
    }

    fn op_sne_vx_vy(&mut self){
        let index_vx = self.get_vx();
        let index_vy = self.get_vy();

        if self.registers[index_vx] != self.registers[index_vy] {
            self.program_counter = self.program_counter + 2;
        }

   }

    fn op_ld_i(&mut self){
        self.indexing = self.opcode & 0x0FFF;
    }

    fn op_jmp_v0(&mut self){
        let offset = (self.opcode & 0x0FFF) as u16;
        self.program_counter = (offset + self.registers[0] as u16) as usize;

    }

    fn op_rnd(&mut self){
        let index_vx = self.get_vx();
        self.registers[index_vx] = rand::random::<u8>() & (self.opcode & 0x00FF) as u8 ;
    }

    fn op_drw(&mut self){
        self.registers[0xF] = 0;

        let initial_col_coord = self.registers[self.get_vx() ];
        let initial_row_coord = self.registers[self.get_vy() ];

        let height:u16 = self.opcode & 0x000F;
        let mut row: u16 = (initial_row_coord % 32) as u16;


        for i in self.indexing..(self.indexing+height){
            
            let sprite_row = self.memory[i as usize];
            
            
            let mut col: u16 = initial_col_coord as u16;

            for j in 0..8{
            
                let pos = (row*64+(col % 64 )) as usize;
                
                let sprite_set = sprite_row & (0x80 >> j); 


                if sprite_set != 0 {
                    if self.gfx[pos] != 0 {
                        self.registers[0xF] = 1;
                    }
                    self.gfx[pos] ^= 1;
                }

                col = col + 1 as u16;
            }

            row = (row + 1) % 32;
            
        }

        
    }
   
    fn op_skp(&mut self){
        let key_index = self.registers[self.get_vx()] as usize;

        if self.keys[key_index] == 1 {
            self.program_counter += 2;
        }
    }
    
    fn op_sknp(&mut self){
        let key_index = self.registers[self.get_vx()] as usize;

        if self.keys[key_index] == 0 {
            self.program_counter += 2;
        }
    }
    
    fn op_ld_vx_dt(&mut self){
        let index = self.get_vx();
        self.registers[index] = self.delay_timer;
    }
    
    fn op_ld_vx_k(&mut self){
        let index = self.get_vx();
        self.wait_index = index as usize;
    }
    
    fn op_ld_dt_vx(&mut self){
        let vx = self.registers[self.get_vx()];
        self.delay_timer = vx;
    }

    fn op_ld_st_vx(&mut self){
        let vx = self.registers[self.get_vx()];
        self.sound_timer = vx;  
    }

    fn op_add_i_vx(&mut self){
        let vx = self.registers[self.get_vx()];
        self.indexing = self.indexing + vx as u16;

    }

    fn op_ld_f_vx(&mut self){
        let sprite_index = self.registers[self.get_vx()] as u16;
        
        self.indexing = ((0x50 +  (sprite_index * 5)) &  0xFFFF ) as u16;
    }

    fn op_ld_b_vx(&mut self){
        let val = self.registers[self.get_vx()];
        let i = self.indexing as usize;

        self.memory[i] = val / 100;
        self.memory[i+1] = (val / 10 ) % 10;
        self.memory[i+2] = val % 10;


    }
    
    fn op_ld_i_vx(&mut self){
        let index_vx = self.get_vx();
        let mem_addr = self.indexing as usize;

        
        for i in 0..index_vx+1{
            self.memory[mem_addr+i]  = self.registers[i as usize];
        }
    }
    
    fn op_ld_vx_i(&mut self){
        let index_vx = self.get_vx();
        let mem_addr = self.indexing as usize;

        for i in 0..index_vx+1{
            self.registers[i]  = self.memory[i + mem_addr];
        }
    }
 
    fn add_with_carry(&self, a:u8, b:u8) -> (u8,u8){
        let s = (a as u16 + b as u16) & 0xFF ;
        let c = if s & 0xFF00 != 0 {1u8} else {0u8};
        
        (s as u8, c)

    }

    fn get_byte(&self) -> u8 {
        (self.opcode & 0x00FF) as u8
    }

    fn get_vy(&self) -> usize {
        ((self.opcode & 0x00F0) >> 4) as usize
    }

    fn get_vx(&self) -> usize {
        ((self.opcode & 0x0F00) >> 8) as usize
    }
   
    pub fn get_gfx(&self) -> &[u8; 64*32]{
        &self.gfx
    }
   
    pub fn update_keys(&mut self, input: char) {
        for i in 0.. self.keys.len(){
            self.keys[i] = 0;
        }

        let keystroke = match input{
            '1' => 1,
            '2' => 2,
            '3' => 3,
            '4' => 0xC,
            'q' => 4,
            'w' => 5,
            'e' => 6,
            'r' => 0xD,
            'a' => 7,
            's' => 8,
            'd' => 9,
            'f' => 0xE,
            'y' => 0xA,
            'x' => 0,
            'c' => 0xB,
            'v' => 0xF,
             _ => 255, 
        };

        if keystroke == 255 {
            return;
        }

        self.keys[keystroke] = 1;

        if self.wait_index != 255 {
            self.registers[self.wait_index] = keystroke as u8;
            self.wait_index = 255;
        }

    }
}
