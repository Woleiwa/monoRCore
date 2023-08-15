#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::get_time;

#[no_mangle]
pub fn main() -> i32 {
    let start = get_time();
    println!("I am sjf2");
    println!("current time_msec = {}", start);
    let mut i = 0; 
    while i < 10000000 {
        i+= 1;
    }
    let end = get_time();
    println!("sjf2 end!");
    println!(
        "time_msec = {}, delta = {}ms, sjf2 OK!",
        end,
        end - start
    );
    0
}
