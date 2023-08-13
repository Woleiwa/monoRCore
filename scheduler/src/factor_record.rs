use alloc::collections:: BTreeMap;

use crate::{Record, time_record_map::RecordMap};

pub struct FactorRecord {
    exp_time: usize,
    factor: usize,
}

impl FactorRecord {
    pub fn new() -> FactorRecord {
        FactorRecord {
            exp_time: 0,
            factor: 10,
        }
    }

    pub fn set_factor(&mut self, new_factor:usize){
        if new_factor <= 0 || new_factor > 100{
            panic!("Invalid factor!");
        }
        self.factor = new_factor;
    }

    pub fn copy(&self)->FactorRecord{
        FactorRecord{
            exp_time: self.exp_time,
            factor: self.factor,
        }
    }
}

impl Record for FactorRecord {
    fn update(&mut self, new_time: usize) {
        if new_time <= 0 {
            return;
        }
        if self.exp_time == 0 {
            self.exp_time = new_time;
        } else {
            let new_exp_time = (100 - self.factor) * self.exp_time + self.factor * new_time;
            self.exp_time = new_exp_time / 100;
        }
    }

    fn get_time(&self) -> usize {
        return self.exp_time;
    }
}

pub struct FactorRecordMap<FactorRecord>{
    map: BTreeMap<usize,FactorRecord>
}

impl <FactorRecord>FactorRecordMap<FactorRecord> {
    pub fn new()-> Self{
        Self { map: BTreeMap::new() }
    }
}

impl <FactorRecord>RecordMap<FactorRecord> for FactorRecordMap<FactorRecord>{
    #[inline]
    fn insert(&mut self, proc:usize, record:FactorRecord) {
        self.map.insert(proc, record);
    }

    #[inline]
    fn get_record(&mut self, proc:usize) ->Option<&mut FactorRecord> {
        return self.map.get_mut(&proc);
    }
}