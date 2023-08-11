pub trait Record {
    fn update(&mut self, new_time: isize); //更新记录时间

    fn get_time(&self) -> isize; //获取预测时间
}
