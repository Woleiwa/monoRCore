use alloc::collections::VecDeque;
use alloc::vec::Vec;
use alloc::vec;
use kernel_vm::{AddressSpace, VmMeta, PageManager, VPN, Pte, PPN};

use crate::frame_allocator::FrameTracker;
use crate::ACCESS_FLAG;

pub struct LruQueue<Meta: VmMeta> {
    pub inner: VecDeque<(PPN<Meta>, VPN<Meta>, FrameTracker, usize)>,
}

impl <Meta: VmMeta> LruQueue<Meta> {
    fn get_pte<M: PageManager<Meta>>(memory_set: &mut AddressSpace<Meta, M>, vpn: &VPN<Meta>) -> Option<Pte<Meta>> {
        memory_set.translate_to_pte(vpn.base())
    }

    fn has_accessed<M: PageManager<Meta>>(memory_set: &mut AddressSpace<Meta, M>, vpn: &VPN<Meta>) -> bool {
        let pte = Self::get_pte(memory_set, vpn).unwrap();
        let flags = pte.flags();
        (flags.val() & ACCESS_FLAG) != 0 
    }

    fn clear_accessed<M: PageManager<Meta>>(memory_set: &mut AddressSpace<Meta, M>, vpn: &VPN<Meta>) {
        memory_set.clear_accessed(*vpn);
    }
}

impl <Meta: VmMeta> LruQueue<Meta> {
    pub fn new() -> Self {
        Self { inner: VecDeque::new()}
    }

    pub fn push_back(&mut self, item: (PPN<Meta>, VPN<Meta>, FrameTracker,usize)){
        Self.inner.push_back(item);
    }

    pub fn len(&self) -> usize{
        self.inner.len()
    }

    pub fn work<M: PageManager<Meta>>(&mut self, memory_set: &mut AddressSpace<Meta, M>) -> (PPN<Meta>, VPN<Meta>) {
        let length = self.inner.len();
        for i in 0..length {
            let (ppn,vpn,_,flag) = &self.inner[i];
            let mut cur_flag = flag;
            let accessed = Self::has_accessed(memory_set,vpn);
            if(accessed){
                cur_flag |= (1 << 8); 
            }
            &self.inner.insert(i, (ppn,vpn,frame,cur_flag));
        }//update flags

        let mut index = 0;
        let (p,v,f,cur) = &self.inner[index];
        let mut minimum = cur;
        for i in 1..length {
            let (ppn,vpn,_,flag) = &self.inner[i];
            if minimum < flag {
                minimum = flag;
                index = i;
            }
        }//find the minimum and remove it
        let (ppn,vpn,_,_) = self.inner.remove(index).unwrap();
        (ppn,vpn)
    }

    pub fn handle_clock_interrupt(&mut self, memory_set: &mut AddressSpace<Meta, M>){
        let length = self.inner.len();
        for i in 0..length {
            let (ppn,vpn,frame,flag) = &self.inner[i];
            let mut cur_flag = flag;
            cur_flag >>= 1;
            let accessed = Self::has_accessed(memory_set,vpn);
            if(accessed){
                cur_flag |= (1 << 8); 
            }
            &self.inner.insert(i, (ppn,vpn,frame,cur_flag));
        }
    }
}