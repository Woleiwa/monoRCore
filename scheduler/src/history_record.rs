use core::option::Option;

use alloc::collections::BTreeMap;

use crate::{Record, time_record_map::RecordMap};
use rcore_console::println;
pub struct HistoryRecord {
    exp_time: usize,
    exec_time: usize,
}

impl HistoryRecord {
    pub fn new() -> HistoryRecord {
        HistoryRecord {
            exp_time: 0,
            exec_time: 0,
        }
    }

    pub fn copy(&self)->HistoryRecord{
        HistoryRecord{
            exp_time: self.exp_time,
            exec_time: self.exec_time,
        }
    }
}

impl Record for HistoryRecord {
    fn update(&mut self, new_time: usize) {
        if new_time <= 0 {
            return;
        }
        println!("new time is {}", new_time);
        let new_exec_time = self.exec_time + 1;
        let total_time = (self.exec_time * self.exp_time + new_time ) / new_exec_time;
        println!("new total_time is {}", total_time);
        self.exec_time =  new_exec_time;
        self.exp_time =  total_time;
    }

    fn get_time(&self) -> usize {
        return self.exp_time;
    }
}

pub struct HistoryRecordMap<HistoryRecord>{
    map: BTreeMap<usize,HistoryRecord>
}

impl <HistoryRecord>HistoryRecordMap<HistoryRecord> {
    pub fn new()-> Self{
        Self { map: BTreeMap::new() }
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