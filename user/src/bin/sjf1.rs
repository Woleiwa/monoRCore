#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::get_time;

#[no_mangle]
pub fn main() -> i32 {
    let start = get_time();
    println!("I am sjf1");
    println!("current time_msec = {}", start);
    let mut i = 0;
    while i < 10000{
        i+= 1;
    }
    let end = get_time();
    println!("sjf1 end!");
    println!(
        "time_msec = {}, delta = {}ms, sjf1 OK!",
        end,
        end - start
    );
    0
}
