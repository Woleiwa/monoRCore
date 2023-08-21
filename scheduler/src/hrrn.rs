use crate::syscall_args::*;
use crate::Manage;
use crate::Schedule;
use alloc::collections::BTreeMap;
use alloc::collections::VecDeque;
use core::cmp::{Ord, Ordering};
use core::convert::TryInto;
use core::marker::Copy;
use factor_record::{FactorRecord, FactorRecordMap};
use history_record::{HistoryRecord, HistoryRecordMap};
use rcore_utils::get_time_ms;
use recorder::Record;
use time_record_map::RecordMap;
use rcore_console:: println;

struct HRRNTaskBlock<I: Copy + Ord> {
    task_id: I,
    time_total: usize,
    time_wait: usize,
    last_stop_time: usize,
    is_ready: bool,
}

impl<I: Copy + Ord> HRRNTaskBlock<I> {
    // pub fn compute_rr(&self, cur_time: usize) -> f64 {
    //     let t = self.time_total;
    //     let mut w = self.time_wait;
    //     if self.last_stop_time != usize::MAX {
    //         w += cur_time - self.last_stop_time;
    //     }
    //     1.0 + (w as f64 / t as f64)
    // }

    pub fn cmp(&self, other: &Self, cur_time: usize) -> Ordering {
        let ts = self.time_total;
        let ws = if self.last_stop_time != usize::MAX {
            (cur_time - self.last_stop_time) + self.time_wait
        } else {
            self.time_wait
        };
        let to = other.time_total;
        let wo = if other.last_stop_time != usize::MAX {
            (cur_time - other.last_stop_time) + other.time_wait
        } else {
            other.time_wait
        };
        (ws * to).cmp(&(wo * ts))
    }

    pub fn update_wait_time(&mut self, cur_time: usize) {
        if self.last_stop_time != usize::MAX {
            self.time_wait += cur_time - self.last_stop_time;
        }
    }
}

pub struct HRRNManager<T, I: Copy + Ord> {
    tasks: BTreeMap<I, T>,
    info_map: BTreeMap<I, HRRNTaskBlock<I>>,
    current: Option<I>, // (id, st_time)
    task_name: BTreeMap<I, usize>,
    running_time: BTreeMap<I, usize>,
    start_time: usize,
    history_record: HistoryRecordMap<HistoryRecord>,
    factor_record: FactorRecordMap<FactorRecord>,
    record_type: bool,
}

impl<T, I: Copy + Ord> HRRNManager<T, I> {
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            info_map: BTreeMap::new(),
            current: None,
            task_name: BTreeMap::new(),
            running_time: BTreeMap::new(),
            start_time: 0,
            history_record: HistoryRecordMap::<HistoryRecord>::new(),
            factor_record: FactorRecordMap::<FactorRecord>::new(),
            record_type: false,
        }
    }
}

impl<T, I: Copy + Ord> HRRNManager<T, I> {
    pub fn get_list(&self) -> VecDeque<(I, usize, usize)> {
        let mut ret: VecDeque<_> = VecDeque::new();
        for (id, block) in self.info_map.iter() {
            ret.push_back((id.clone(), block.time_total, block.time_wait));
        }
        ret
    }
}

impl<T, I: Copy + Ord> Manage<T, I> for HRRNManager<T, I> {
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
        self.info_map.remove(&id);
        if let Some(cur_id) = self.current {
            if cur_id == id {
                self.current = None
            }
        }
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

impl<T, I: Copy + Ord> Schedule<I> for HRRNManager<T, I> {
    fn add(&mut self, id: I) {
        if let Some(block) = self.info_map.get_mut(&id) {
            block.is_ready = true;
        } else {
            self.info_map.insert(
                id,
                HRRNTaskBlock {
                    task_id: id,
                    time_total: 1000_000_000,
                    time_wait: 0,
                    last_stop_time: usize::MAX,
                    is_ready: true,
                },
            );
        }
    }

    fn fetch(&mut self) -> Option<I> {
        let time = get_time_ms();
        let task_block = self
            .info_map
            .iter()
            .filter(|&(_, block)| block.is_ready)
            .max_by(|&a, &b| a.1.cmp(b.1, time));

        match task_block {
            None => None,
            Some((id, _)) => Some(id.clone()),
        }
    }

    fn update_exec(&mut self, id: I, args: &ExecArgs) {
        let record_time = match self.record_type {
            true => {
                let record = self.factor_record.get_record(args.proc);
                let res = match record {
                    None => {
                        println!("No record time for {}", args.proc);
                        args.total_time
                    }
                    Some(record) => {
                        println!("Record time for {} is {}", args.proc, record.get_time());
                        record.get_time()
                    }
                };
                self.task_name.insert(id, args.proc);
                res
            }
            false => {
                let record = self.history_record.get_record(args.proc);
                let res = match record {
                    None => {
                        println!("No record time for {}", args.proc);
                        args.total_time
                    }
                    Some(record) => {
                        println!("Record time for {} is {}", args.proc, record.get_time());
                        record.get_time()
                    }
                };
                self.task_name.insert(id, args.proc);
                res
            }
        };
        let block = self.info_map.get_mut(&id).unwrap();
        block.time_total = record_time;
        block.time_wait = 0;
        block.last_stop_time = usize::MAX;
    }

    fn update_fork(&mut self, parent_id: I, child_id: I) {
        let parent = self.info_map.get(&parent_id).unwrap();
        self.info_map.insert(
            child_id,
            HRRNTaskBlock {
                task_id: child_id,
                time_total: parent.time_total,
                time_wait: parent.time_wait,
                last_stop_time: parent.last_stop_time,
                is_ready: parent.is_ready,
            },
        );
        let proc =  self.task_name.get(&parent_id);
        match proc{
            Some(t) =>{
                self.task_name.insert(child_id, *t);
            }
            None =>{

            }
        }
        let run_time = self.running_time.get(&parent_id);
        match run_time{
            Some(t) =>{
                self.running_time.insert(child_id, *t);
            }
            None =>{
                
            }
        }
    }

    fn update_sched_to(&mut self, id: I, time: usize) {
        self.start_time = time;
        if let None = self.current {
            self.current = Some(id);
            let block = self.info_map.get_mut(&id).unwrap();
            block.update_wait_time(time);
        } else {
            panic!("call sched while current is not suspended!")
        }
    }

    fn update_suspend(&mut self, id: I, time: usize) {
        if let Some(cur_id) = self.current {
            if cur_id != id {
                panic!("suspend wrong id? ");
            }

            self.current = None;

            let block = self.info_map.get_mut(&cur_id).unwrap();
            block.last_stop_time = time;
            block.is_ready = false;

            let run_time = time - self.start_time;
            let cur_time = match self.running_time.get(&id) {
                None => 0,
                Some(t) => *t,
            };
            let total = cur_time + run_time;
            self.running_time.insert(id, total);
        } else {
            panic!("call suspend but current is none! ")
        }
    }

    fn update_sleep(&mut self, id: I) {}
}
