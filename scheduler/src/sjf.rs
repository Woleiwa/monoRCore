use crate::Manage;
use crate::Schedule;
use crate::syscall_args::*;
use alloc::collections::{BTreeMap, BinaryHeap};
use core::cmp::{Ord, Ordering};
use core::convert::TryInto;
use core::marker::Copy;
use core::option::Option;
use factor_record::{FactorRecord, FactorRecordMap};
use history_record::{HistoryRecord, HistoryRecordMap};
use recorder::Record;
use time_record_map::RecordMap;
use rcore_console::println;

struct SJFTaskBlock<I: Copy + Ord> {
    task_id: I,
    time: usize,
}

impl<I: Copy + Ord> PartialOrd for SJFTaskBlock<I> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.time.partial_cmp(&self.time) //reverse order
    }
}

impl<I: Copy + Ord> PartialEq for SJFTaskBlock<I> {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl<I: Copy + Ord> Ord for SJFTaskBlock<I> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.time.cmp(&self.time)
    }
}

impl<I: Copy + Ord> Eq for SJFTaskBlock<I> {}

pub struct SJFManager<T, I: Copy + Ord> {
    tasks: BTreeMap<I, T>,
    task_name: BTreeMap<I, usize>,
    time_map: BTreeMap<I, usize>,
    start_time: usize,
    heap: BinaryHeap<SJFTaskBlock<I>>, // max-heap, reverse Ord to get a min-heap
    running_time: BTreeMap<I, usize>,
    history_record: HistoryRecordMap<HistoryRecord>,
    factor_record: FactorRecordMap<FactorRecord>,
    record_type: bool,
}

impl<T, I: Copy + Ord> SJFManager<T, I> {
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            task_name: BTreeMap::new(),
            time_map: BTreeMap::new(),
            start_time: 0,
            heap: BinaryHeap::new(),
            running_time: BTreeMap::new(),
            history_record: HistoryRecordMap::<HistoryRecord>::new(),
            factor_record: FactorRecordMap::<FactorRecord>::new(),
            record_type: false,
        }
    }
}

impl<T, I: Copy + Ord> Manage<T, I> for SJFManager<T, I> {
    /// 插入一个新任务
    #[inline]
    fn insert(&mut self, id: I, task: T) {
        self.tasks.insert(id, task);
    }
    /// 根据 id 获取对应的任务
    #[inline]
    fn get_mut(&mut self, id: I) -> Option<&mut T> {
        self.tasks.get_mut(&id)
    }
    /// 删除任务实体
    #[inline]
    fn delete(&mut self, id: I) {
        self.tasks.remove(&id);
        self.time_map.remove(&id);
        let task_name = self.task_name.get(&id);
        match task_name {
            None => {}
            Some(task_name) => {
                match self.running_time.get(&id) {
                    None => {return;}
                    Some(t) => {
                        let run_time = *t;
                        self.running_time.remove(&id);
                        match self.record_type {
                            true => {
                                let f_record = &mut self.factor_record;
                                let record = f_record.get_record(*task_name);
                                let new_record = match record {
                                    None => {
                                        let mut new_record = FactorRecord::new();
                                        new_record.update(run_time.try_into().unwrap());
                                        new_record.copy()
                                    }
                                    Some(cur_record) => {
                                        cur_record.update(run_time.try_into().unwrap());
                                        cur_record.copy()
                                    }
                                };
                                println!("new time for {} is {}", task_name, new_record.get_time());
                                f_record.insert(*task_name, new_record);
                            }
                            false => {
                                let h_record = &mut self.history_record;
                                let record = h_record.get_record(*task_name);
                                let new_record = match record {
                                    None => {
                                        let mut new_record = HistoryRecord::new();
                                        new_record.update(run_time.try_into().unwrap());
                                        new_record.copy()
                                    }
                                    Some(cur_record) => {
                                        cur_record.update(run_time.try_into().unwrap());
                                        cur_record.copy()
                                    }
                                };
                                println!("new time for {} is {}", task_name, new_record.get_time());
                                h_record.insert(*task_name, new_record);
                            }
                        };
                    }
                }
                self.task_name.remove(&id);
            }
        }
    }
}

impl<T, I: Copy + Ord> Schedule<I> for SJFManager<T, I> {
    fn add(&mut self, id: I) {
        let time_ = match self.time_map.get(&id) {
            None => isize::MAX as usize,
            Some(t) => *t,
        };
        self.heap.push(SJFTaskBlock {
            task_id: id,
            time: time_,
        });
        self.time_map.insert(id, time_);
    }

    fn fetch(&mut self) -> Option<I> {
        match self.heap.pop() {
            None => None,
            Some(tb) => Some(tb.task_id),
        }
    }

    fn update_exec(&mut self, id: I, args: &ExecArgs) {
        match self.record_type {
            true => {
                let record = self.factor_record.get_record(args.proc);
                match record {
                    None => {
                        self.time_map.insert(id, args.time);
                        println!("No record time for {}", args.proc);
                    }
                    Some(record) => {
                        self.time_map.insert(id, record.get_time().try_into().unwrap());
                        println!("Record time for {} is {}", args.proc, record.get_time());
                    }
                }
                self.task_name.insert(id, args.proc);
            }
            false => {
                let record = self.history_record.get_record(args.proc);
                match record {
                    None => {
                        self.time_map.insert(id, args.time);
                        println!("No record time for {}", args.proc);
                    }
                    Some(record) => {
                        self.time_map.insert(id, record.get_time().try_into().unwrap());
                        println!("Record time for {} is {}", args.proc, record.get_time());
                    }
                }
                self.task_name.insert(id, args.proc);
            }
        };
    }

    fn update_fork(&mut self, parent_id: I, child_id: I) {
        let time_ = match self.time_map.get(&parent_id) {
            None => isize::MAX as usize,
            Some(t) => *t,
        };
        let proc =  self.task_name.get(&parent_id);
        match proc{
            Some(t) =>{
                self.task_name.insert(child_id, *t);
            }
            None =>{
                
            }
        }
        /*let run_time = self.running_time.get(&parent_id);
        match run_time{
            Some(t) =>{
                self.running_time.insert(child_id, *t);
            }
            None =>{
                
            }
        }*/
        self.time_map.insert(child_id, time_);
    }

    fn update_sched_to(&mut self, id: I, time: usize) {
        self.start_time = time;
    }

    fn update_suspend(&mut self, id: I, time: usize) {
        let run_time = time - self.start_time;
        let cur_time = match self.running_time.get(&id) {
            None => 0,
            Some(t) => *t,
        };
        let total = cur_time + run_time;
        self.running_time.insert(id, total);
    }

    fn update_sleep(&mut self, id: I) {}
}
