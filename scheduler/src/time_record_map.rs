use alloc::string::String;
use core::option::Option;
pub trait RecordMap<I> {
    fn insert(&mut self, proc:String, record:I);//插入时间记录

    fn get_record(&mut self, proc:String) ->Option<&mut I>;//获取时间记录
}


