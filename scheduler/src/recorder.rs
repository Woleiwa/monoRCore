pub trait Record {
    fn update(&mut self, new_time: usize); //更新记录时间

    fn get_time(&self) -> usize; //获取预测时间
}
