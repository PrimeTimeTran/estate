pub trait Focus {
    fn start(&mut self) -> io::Result<()>;
    fn stop(&mut self);
}