use core::marker::PhantomData;

use crate::plugins::{handle_pagefault, Manage};
use crate::frame_allocator::{FrameTracker, frame_check};
use kernel_vm::{AddressSpace, VmMeta, PageManager, VPN, PPN};
use alloc::vec::Vec;
use alloc::vec;
use crate::lru_queue::LruQueue;


pub struct LRUManager<Meta:VmMeta,M:PageManager<Meta>> {
    queue: BTreeMap<usize, LruQueue<Meta>>,
    manager: PhantomData<M>
}

impl<Meta: VmMeta, M: PageManager<Meta> + 'static> Manage<Meta, M> for LRUManager<Meta, M> {
    fn new() -> Self {
        Self { queue: BTreeMap::new(), manager: PhantomData }
    }

    fn handle_pagefault<F>(&mut self, get_memory_set: &F, vpn: VPN<Meta>, task_id: usize)
            where F: Fn(usize) -> &'static mut AddressSpace<Meta, M> {
        handle_pagefault(get_memory_set, vpn, task_id, self)
    } 

    fn insert_frame(&mut self, vpn: VPN<Meta>, ppn: PPN<Meta>, task_id: usize, frame: FrameTracker) {
       let flag = 1 << 8;
       if let Some(vec) = self.queue.get_mut(&task_id) {
            vec.push_back((ppn, vpn, frame, flag));
        } else {
            let mut tmp = ClockQueue::new();
            tmp.push_back((ppn, vpn, frame, flag));
            self.queue.insert(task_id, tmp);
        }
    }

    fn work<F>(&mut self, get_memory_set: &F, task_id: usize) -> Vec<(PPN<Meta>, VPN<Meta>, usize)>  
        where F: Fn(usize) -> &'static mut AddressSpace<Meta, M> {
        if !frame_check() {
            let memory_set = get_memory_set(task_id);
            let queue = self.queue.get_mut(&task_id).unwrap();
            assert!(queue.len() != 0);
            let item = queue.work(memory_set);
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
            let memory_set = get_memory_set(task_id);
            for (key,value) in self.queue{
                let vec = self.queue.get_mut(&key).unwrap;
                vec.handle_clock_interrupt(memory_set);
            }
        }
    }
}