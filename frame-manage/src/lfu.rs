use core::marker::PhantomData;
use alloc::collections::BTreeMap;
use crate::plugins::{handle_pagefault, Manage};
use crate::frame_allocator::{FrameTracker, frame_check};
use kernel_vm::{AddressSpace, VmMeta, PageManager, VPN, PPN};
use alloc::vec::Vec;
use alloc::vec;
use crate::lfu_queue::LfuQueue;


pub struct LFUManager<Meta:VmMeta,M:PageManager<Meta>> {
    queue: BTreeMap<usize, LfuQueue<Meta>>,
    manager: PhantomData<M>
}

impl<Meta: VmMeta, M: PageManager<Meta> + 'static> Manage<Meta, M> for LFUManager<Meta, M> {
    fn new() -> Self {
        Self { queue: BTreeMap::new(), manager: PhantomData }
    }

    fn handle_pagefault<F>(&mut self, get_memory_set: &F, vpn: VPN<Meta>, task_id: usize)
            where F: Fn(usize) -> &'static mut AddressSpace<Meta, M> {
        handle_pagefault(get_memory_set, vpn, task_id, self)
    } 

    fn insert_frame(&mut self, vpn: VPN<Meta>, ppn: PPN<Meta>, task_id: usize, frame: FrameTracker) {
       let flag = 1;
       if let Some(vec) = self.queue.get_mut(&task_id) {
            vec.push_back((ppn, vpn, frame, flag));
        } else {
            let mut tmp = LfuQueue::new();
            tmp.push_back((ppn, vpn, frame, flag));
            self.queue.insert(task_id, tmp);
        }
    }

    fn work<F>(&mut self, get_memory_set: &F, task_id: usize) -> Vec<(PPN<Meta>, VPN<Meta>, usize)>  
        where F: Fn(usize) -> &'static mut AddressSpace<Meta, M> {
        if !frame_check() {
            let memory_set = get_memory_set(task_id);
            let cur_queue = self.queue.get_mut(&task_id).unwrap();
            assert!(cur_queue.len() != 0);
            let item = cur_queue.work(memory_set);
            vec![(item.0, item.1, task_id)]
        } else {
            vec![]
        }
    }

    fn clear_frames(&mut self, task_id: usize) {
        self.queue.remove(&task_id);
    }

    fn handle_time_interrupt<F>(&mut self, get_memory_set: &F) 
            where F: Fn(usize) -> &'static mut AddressSpace<Meta, M> {
        if !frame_check() {
            for (key,cur_queue)in self.queue.iter_mut(){
                let memory_set = get_memory_set(*key);
                cur_queue.handle_clock_interrupt(memory_set);
            }
        }
    }
}