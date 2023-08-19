use core::marker::PhantomData;
use rand_chacha::rand_core::block;
use crate::frame_allocator::{frame_check, FrameTracker};
use crate::plugins::{handle_pagefault, Manage};
use alloc::collections::{BTreeMap, VecDeque};
use alloc::vec;
use alloc::vec::Vec;
use kernel_vm::{AddressSpace, PageManager, VmMeta, PPN, VPN};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

pub struct NRUManager<Meta: VmMeta, M: PageManager<Meta>> {
    queue: BTreeMap<usize, VecDeque<(PPN<Meta>, VPN<Meta>, FrameTracker)>>,
    manager: PhantomData<M>,
    rng: ChaCha20Rng,
}

impl<Meta: VmMeta, M: PageManager<Meta>> NRUManager<Meta, M> {
    fn get_pte<M: PageManager<Meta>>(
        memory_set: &mut AddressSpace<Meta, M>,
        vpn: &VPN<Meta>,
    ) -> Option<Pte<Meta>> {
        memory_set.translate_to_pte(vpn.base())
    }

    fn has_accessed<M: PageManager<Meta>>(
        memory_set: &mut AddressSpace<Meta, M>,
        vpn: &VPN<Meta>,
    ) -> bool {
        let pte = Self::get_pte(memory_set, vpn).unwrap();
        let flags = pte.flags();
        (flags.val() & ACCESS_FLAG) != 0
    }

    fn clear_accessed<M: PageManager<Meta>>(
        memory_set: &mut AddressSpace<Meta, M>,
        vpn: &VPN<Meta>,
    ) {
        memory_set.clear_accessed(*vpn);
    }

    fn get_accessed_dirty<M: PageManager<Meta>>(
        memory_set: &mut AddressSpace<Meta, M>,
        vpn: &VPN<Meta>,
    ) -> (bool, bool) {
        let pte = Self::get_pte(memory_set, vpn).unwrap();
        let flags = pte.flags().val();
        ((flags & ACCESS_FLAG) != 0, (flags & DIRTY_FLAG) != 0)
    }
}

impl<Meta: VmMeta, M: PageManager<Meta> + 'static> Manage<Meta, M> for NRUManager<Meta, M> {
    fn new() -> Self {
        Self {
            queue: BTreeMap::new(),
            manager: PhantomData,
            rng: ChaCha20Rng::seed_from_u64(233u64)
        }
    }

    fn handle_pagefault<F>(&mut self, get_memory_set: &F, vpn: VPN<Meta>, task_id: usize)
    where
        F: Fn(usize) -> &'static mut AddressSpace<Meta, M>,
    {
        handle_pagefault(get_memory_set, vpn, task_id, self)
    }

    fn insert_frame(
        &mut self,
        vpn: VPN<Meta>,
        ppn: PPN<Meta>,
        task_id: usize,
        frame: FrameTracker,
    ) {
        if let Some(vec) = self.queue.get_mut(&task_id) {
            vec.push_back((ppn, vpn, frame));
        } else {
            let mut tmp = VecDeque::new();
            tmp.push_back((ppn, vpn, frame));
            self.queue.insert(task_id, tmp);
        }
    }

    fn work<F>(&mut self, get_memory_set: &F, task_id: usize) -> Vec<(PPN<Meta>, VPN<Meta>, usize)>
    where
        F: Fn(usize) -> &'static mut AddressSpace<Meta, M>,
    {
        if !frame_check() {
            let list = self.queue.get_mut(&task_id).unwrap();
            let memory_set = get_memory_set(task_id);
            let length = list.len();
            let index_of_ad = VecDeque<usize>::new();//accessed and dirty
            let index_of_a = VecDeque<usize>::new();//only accessed
            let index_of_d =  VecDeque<usize>::new();//only dirty
            let index_of_n = VecDeque<usize>::new();//neither accessed nor dirty
            for i in 0..length{
                let (_ppn,vpn,_frame,flag) = list[i];
                let (accessed,dirty) = Self::get_accessed_dirty(memory_set, vpn);
                if accessed && dirty {
                    index_of_ad.push_back(i);
                }
                else if accessed {
                    index_of_a.push_back(i);
                }
                else if dirty {
                    index_of_d.push_back(i);
                }
                else {
                    index_of_n.push_back(i);
                }
            }
            VecDeque<VecDeque<usize>> indexes;
            indexes.push_back(index_of_n);
            indexes.push_back(index_of_d);
            indexes.push_back(index_of_a);
            indexes.push_back(index_of_ad);
            let mut removed_index = 0;
            for i in 0..4{
                let index = indexes[i];
                if index.len() != 0 {
                    let lucky_dog = self.rng.gen_range(0..index.len());
                    removed_index = index[lucky_dog];
                }
            }
            let entry = list.remove(removed_index).unwrap();
            vec![(entry.0, entry.1, task_id)]
        } else {
            vec![]
        }
    }

    fn clear_frames(&mut self, task_id: usize) {
        self.queue.remove(&task_id);
    }

    fn handle_time_interrupt<F>(&mut self, get_memory_set: &F)
    where
        F: Fn(usize) -> &'static mut AddressSpace<Meta, M>,
    {
        if !frame_check() {
            for (key, cur_queue) in self.queue.iter_mut() {
                let memory_set = get_memory_set(*key);
                let length = cur_queue.len();
                for i in 0..length {
                    let (_ppn,vpn,_frame,flag) = cur_queue[i];
                    Self::clear_accessed(memory_set, vpn);
                }
            }
        }
    }
}
