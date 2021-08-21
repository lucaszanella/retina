pub trait EncodedPacket<'a, T: Send>: Send {
    fn get_type(&self) -> PacketType;
    fn channel_id(&self) -> u8;
    fn data(&self) -> MyRef<'_, 'a, [T]>;
    fn data_mut(&mut self) -> Result<MyMut<'_, 'a, [T]>, PacketError>;
    fn to_owned(&self) -> Box<dyn EncodedPacket<'static, T>>;
    fn resize_owned_data(&mut self, size: usize, fill_with: T) -> Result<(), PacketError>;
}