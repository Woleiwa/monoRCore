use alloc::collections::{BinaryHeap, BTreeMap};
use core::marker::Copy;
use core::cmp::{Ord, Ordering};
use core::option::Option;
use crate::Manage;
use crate::Schedule;
use crate::syscall_args::*;
use factor_record;
use history_record;

struct SJFTaskBlock<I: Copy + Ord> {
    task_id: I,
    time: usize
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
    task_name: BTreeMap<I, String>,
    time_map: BTreeMap<I, usize>,
    start_time: usize,
    heap: BinaryHeap<SJFTaskBlock<I>>, // max-heap, reverse Ord to get a min-heap
    history_record: HistoryRecordMap,
    factor_record: FactorRecordMap,
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
            history_record: HistoryRecordMap::new(),
            factor_record: FactorRecordMap::new(),
            record_type: false,
        }
    }
}

impl<T, I: Copy + Ord> Manage<T, I> for SJFManager<T, I>{
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
        self.task_name.remove(&id);
    }
}

impl<T, I: Copy + Ord> Schedule<I> for SJFManager<T, I> {
    fn add(&mut self, id: I) {
        let time_ = match self.time_map.get(&id) {
            None => isize::MAX as usize,
            Some(t) => *t
        };
        self.heap.push(SJFTaskBlock {
            task_id: id,
            time: time_
        });
        self.time_map.insert(id, time_);
    }

    fn fetch(&mut self) -> Option<I> {
        match self.heap.pop() {
            None => None,
            Some(tb) => Some(tb.task_id)
        }
    }

    fn update_exec(&mut self, id: I, args: &ExecArgs) {
        
        let record = match self.record_type{
            true=>{self.factor_record.get_record(args.proc)}
            false=>{self.history_record.get_record(args.proc)}
        };
        match record {
            None => {self.time_map.insert(id, args.time);}
            Some => {self.time_map.insert(id, record.get_time());}
        }
        self.task_name.insert(id, args.proc);
    }

    fn update_fork(&mut self, parent_id: I, child_id: I) {
        let time_ = match self.time_map.get(&parent_id) {
            None => isize::MAX as usize,
            Some(t) => *t
        };
        self.time_map.insert(child_id, time_);
    }

    fn update_sched_to(&mut self, id: I, time: usize) {
        self.start_time = time;
    }
    
    fn update_suspend(&mut self, id: I, time: usize) {
        let running_time = time - start_time;
        let task_name = task_name.get(id);
        let record = match self.record_type{
            true=>{self.factor_record.get_record(task_name)}
            false=>{self.history_record.get_record(task_name)}
        };
        
        match record {
            None => {
                match self.record_type{
                    true=>{
                        let mut record = FactorRecord::new();
                        record.update(running_time);
                        self.factor_record.insert(task_name, running_time);
                    }
                    false=>{
                        let mut record = HistoryRecord::new();
                        record.update(running_time);
                        self.history_record.insert(task_name, running_time);
                    }
                };
            }
            Some => {
                match self.record_type{
                    true=>{
                        record.update(running_time);
                        self.factor_record.insert(task_name, running_time);
                    }
                    false=>{
                        record.update(running_time);
                        self.history_record.insert(task_name, running_time);
                    }
                };
            }
        }
    }

    fn update_sleep(&mut self, id: I) {}
}