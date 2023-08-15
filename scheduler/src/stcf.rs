use alloc::collections::{BinaryHeap, BTreeMap, VecDeque};
use core::marker::Copy;
use core::cmp::{Ord, Ordering};
use core::option::Option;
use crate::Manage;
use crate::Schedule;
use crate::syscall_args::*;
use factor_record::{FactorRecord, FactorRecordMap};
use history_record::{HistoryRecord, HistoryRecordMap};
use recorder::Record;
use time_record_map::RecordMap;
use core::convert::TryInto;
use rcore_console:: println;
struct STCFTaskBlock<I: Copy + Ord> {
    task_id: I,
    time_total: isize,
    time_left: isize
}

impl<I: Copy + Ord> PartialOrd for STCFTaskBlock<I> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.time_left != other.time_left {
            other.time_left.partial_cmp(&self.time_left)
        } else {
            self.task_id.partial_cmp(&other.task_id)
        }
    }
}

impl<I: Copy + Ord> PartialEq for STCFTaskBlock<I> {
    fn eq(&self, other: &Self) -> bool {
        self.task_id == other.task_id
    }
}

impl<I: Copy + Ord> Ord for STCFTaskBlock<I> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.time_left != other.time_left {
            other.time_left.cmp(&self.time_left)
        } else {
            self.task_id.cmp(&other.task_id)
        }
    }
}

impl<I: Copy + Ord> Eq for STCFTaskBlock<I> {}

pub struct STCFManager<T, I: Copy + Ord> {
    tasks: BTreeMap<I, T>,
    time_map: BTreeMap<I, (isize, isize)>, // (time_total, time_left)
    heap: BinaryHeap<STCFTaskBlock<I>>,
    current: Option<(I, usize)>,  // (id, st_time)
    task_name: BTreeMap<I, usize>,
    history_record: HistoryRecordMap<HistoryRecord>,
    factor_record: FactorRecordMap<FactorRecord>,
    record_type: bool
}

impl<T, I: Copy + Ord> STCFManager<T, I> {
    pub fn new() -> Self {
        Self { 
            tasks: BTreeMap::new(), 
            time_map: BTreeMap::new(), 
            heap: BinaryHeap::new(),
            current: None,
            task_name: BTreeMap::new(),
            history_record: HistoryRecordMap::<HistoryRecord>::new(),
            factor_record: FactorRecordMap::<FactorRecord>::new(),
            record_type: false,
        }
    }

    // fn update_total_time(&mut self, id: &I, new_time: isize) {
    //     if new_time <= 0 {
    //         panic!("total time can't be negative or zero! ");
    //     }

    //     if let Some((total_, left_)) = self.time_map.get(id) {
    //         let done = total_ - left_;
    //         let mut new_left = new_time - done;
    //         new_left = if new_left < 0 { 0 } else { new_left };
    //         self.time_map.insert(id.clone(), (new_time, new_left));
    //     } else {
    //         // not registered
    //         self.time_map.insert(id.clone(), (new_time, new_time));
    //     }
    // }

    fn update_left_time(&mut self, id:I, time_pass: isize) {
        match self.time_map.get(&id) {
            None => { panic!("try to update left time for non-exist!"); },
            Some(&(total_, old_time)) => {
                let new_time = old_time - time_pass;
                self.time_map.insert(id, (total_, new_time));
            }
        }  
    }
}

impl<T, I: Copy + Ord> STCFManager<T, I> {
    pub fn get_list(&self) -> VecDeque<(I, isize, isize)> {
        let ret: VecDeque<(I, isize, isize)> = self.heap.iter().map(|block| (block.task_id, block.time_total, block.time_left)).collect();
        ret
    }
}

impl<T, I: Copy + Ord> Manage<T, I> for STCFManager<T, I>{
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
        
        if let Some((cur_id, _)) = self.current {
            if cur_id == id {
                self.current = None
            }
        }
        let task_name = self.task_name.get(&id);
        match task_name {
            None => {
            }
            Some(task_name) => {
                let running_time:isize = match self.time_map.get(&id){
                    None=>{ 
                        0
                    }
                    Some(&(total_time, time_left)) =>{
                        let res = total_time - time_left;
                        res
                    }
                };
                println!("running time for {} is {}" , task_name, running_time );
                match self.record_type {
                    true => {
                        let f_record = &mut self.factor_record;
                        let record = f_record.get_record(*task_name);
                        let new_record = match record {
                            None => {
                                let mut new_record = FactorRecord::new();
                                new_record.update(running_time.try_into().unwrap());
                                new_record.copy()
                            }
                            Some(cur_record) => {
                                cur_record.update(running_time.try_into().unwrap());
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
                                new_record.update(running_time.try_into().unwrap());
                                new_record.copy()
                            }
                            Some(cur_record) => {
                                cur_record.update(running_time.try_into().unwrap());
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
        self.time_map.remove(&id);
    }
}

impl<T, I: Copy + Ord> Schedule<I> for STCFManager<T, I> {
    fn add(&mut self, id: I) {
        let (total_, left_) = match self.time_map.get(&id) {
            None => (isize::MAX, isize::MAX),
            Some(t) => *t
        };
        self.heap.push(STCFTaskBlock { 
            task_id: id, 
            time_total: total_, 
            time_left: left_ 
        });
        self.time_map.insert(id, (total_, left_));
    }

    fn fetch(&mut self) -> Option<I> {
        match self.heap.pop() {
            None => None,
            Some(tb) => Some(tb.task_id)
        }
    }

    fn update_exec(&mut self, id: I, args: &ExecArgs) {
        match self.record_type {
            true => {
                let record = self.factor_record.get_record(args.proc);
                match record {
                    None => {
                        self.time_map.insert(id, (args.total_time, args.total_time));
                        println!("No record time for {}", args.proc);
                    }
                    Some(record) => {
                        self.time_map.insert(id, (record.get_time().try_into().unwrap(),record.get_time().try_into().unwrap()));
                        println!("Record time for {} is {}", args.proc, record.get_time());
                    }
                }
                self.task_name.insert(id, args.proc);
            }
            false => {
                let record = self.history_record.get_record(args.proc);
                match record {
                    None => {
                        self.time_map.insert(id, (args.total_time, args.total_time));
                        println!("No record time for {}", args.proc);
                    }
                    Some(record) => {
                        self.time_map.insert(id, (record.get_time().try_into().unwrap(),record.get_time().try_into().unwrap()));
                        println!("Record time for {} is {}", args.proc, record.get_time());
                    }
                }
                self.task_name.insert(id, args.proc);
            }
        };
    }

    fn update_fork(&mut self, parent_id: I, child_id: I) {
        let time_pair = match self.time_map.get(&parent_id) {
            None => (isize::MAX, isize::MAX),
            Some(&t) => t
        };
        let proc =  self.task_name.get(&parent_id);
        match proc{
            Some(t) =>{
                self.task_name.insert(child_id, *t);
            }
            None =>{

            }
        }
        self.time_map.insert(child_id, time_pair);
    }

    fn update_sched_to(&mut self, id: I, time: usize) {
        if let None = self.current {
            self.current = Some((id, time));
        } else {
            panic!("call sched while current is not suspended!")
        }
    }

    fn update_suspend(&mut self, id: I, time: usize) {
        if let Some((cur_id, st_time)) = self.current {
            if cur_id != id {
                panic!("suspend wrong id? ");
            }

            self.current = None;
            let time_pass: usize = time - st_time;
            self.update_left_time(id, time_pass as isize)
        } else {
            panic!("call suspend but current is none! ")
        }
    }

    fn update_sleep(&mut self, id: I) {}
}