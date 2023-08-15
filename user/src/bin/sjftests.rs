#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;
use alloc::string::String;
static TESTS: &[&str] = &[
    "sjf1",
    "sjf2",
    "sjf3",
    "sjf4",
    "sjf5",
];

static TIMES: [usize;5] = [
    1000,
    10000,
    10000,
    3000,
    1000,
];

use user_lib::{exec_with_args, fork, sleep_noblock, get_time};

#[no_mangle]
pub fn main() -> i32 {
    let mut i = 0;
    for test in TESTS { 
        if i == 3 || i == 4{
            sleep_noblock(1000);
        }
        let start = get_time();
        println!("{} Arrive at {}", test, start);
        let pid = fork();
        if pid == 0 {
            let name = i;
            exec_with_args(*test, (&(TIMES[i],name)) as *const _ as usize);
            panic!("unreachable!");
        }
        i += 1;
    }
    sleep_noblock(100000);
    i = 0;
    for test in TESTS { 
        if i == 3 || i == 4{
            sleep_noblock(1000);
        }
        let start = get_time();
        println!("{} Arrive at {}", test, start);
        let pid = fork();
        if pid == 0 {
            let name = i;
            exec_with_args(*test, (&(TIMES[i],name)) as *const _ as usize);
            panic!("unreachable!");
        }
        i += 1;
    }
    0
}
