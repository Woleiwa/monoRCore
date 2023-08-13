
use core::option::Option;
pub trait RecordMap<I> {
    fn insert(&mut self, proc:usize, record:I);//插入时间记录

    fn get_record(&mut self, proc:usize) ->Option<&mut I>;//获取时间记录
}


