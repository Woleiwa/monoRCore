use std::collections::HashMap;
use alloc::string::String;
use core::option::Option;

use crate::{Record, time_record_map::RecordMap};
pub struct HistoryRecord {
    exp_time: isize,
    exec_time: isize,
}

impl HistoryRecord {
    pub fn new() -> HistoryRecord {
        HistoryRecord {
            exp_time: 0,
            exec_time: 0,
        }
    }
}

impl Record for HistoryRecord {
    fn update(&mut self, new_time: isize) {
        if new_time <= 0 {
            panic!("Invalid time!");
        }
        let new_exec_time = self.exec_time + 1;
        let total_time = (self.exec_time * self.exp_time + new_time ) / new_exec_time;
        self.exec_time =  new_exec_time;
        self.exp_time =  total_time;
    }

    fn get_time(&self) -> isize {
        return self.exec_time;
    }
}

pub struct HistoryRecordMap<HistoryRecord>{
    map: HashMap<usize,HistoryRecord>
}

impl <HistoryRecord>HistoryRecordMap<HistoryRecord> {
    pub fn new()-> Self{
        Self { map: HashMap::new() }
    }
}

impl <HistoryRecord>RecordMap<HistoryRecord> for HistoryRecordMap<HistoryRecord>{
    #[inline]
    fn insert(&mut self, proc:usize, record:HistoryRecord) {
        self.map.insert(proc, record);
    }

    #[inline]
    fn get_record(&mut self, proc:usize) ->Option<&mut HistoryRecord> {
       return self.map.get_mut(&proc);
    }
}