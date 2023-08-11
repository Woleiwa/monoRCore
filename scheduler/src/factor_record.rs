use std::collections::HashMap;
use alloc::string::String;

use crate::{Record, time_record_map::RecordMap};

pub struct FactorRecord {
    exp_time: isize,
    factor: isize,
}

impl FactorRecord {
    pub fn new() -> FactorRecord {
        FactorRecord {
            exp_time: 0,
            factor: 10,
        }
    }

    pub fn set_factor(&mut self, new_factor:isize){
        if new_factor <= 0 || new_factor > 100{
            panic!("Invalid factor!");
        }
        self.factor = new_factor;
    }
}

impl Record for FactorRecord {
    fn update(&mut self, new_time: isize) {
        if new_time <= 0 {
            panic!("Invalid time!");
        }
        if self.exp_time == 0 {
            self.exp_time = new_time;
        } else {
            let new_exp_time = (100 - self.factor) * self.exp_time + self.factor * new_time;
            self.exp_time = new_exp_time / 100;
        }
    }

    fn get_time(&self) -> isize {
        return self.exp_time;
    }
}

pub struct FactorRecordMap<FactorRecord>{
    map: HashMap<String,FactorRecord>
}

impl <FactorRecord>FactorRecordMap<FactorRecord> {
    pub fn new()-> Self{
        Self { map: HashMap::new() }
    }
}

impl <FactorRecord>RecordMap<FactorRecord> for FactorRecordMap<FactorRecord>{
    #[inline]
    fn insert(&mut self, proc:String, record:FactorRecord) {
        self.map.insert(proc, record);
    }

    #[inline]
    fn get_record(&mut self, proc:String) ->Option<&mut FactorRecord> {
        return self.map.get_mut(&proc);
    }
}