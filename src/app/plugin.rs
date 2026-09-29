trait Plugin {
    fn activate(&self, ctx: &Context) -> Result<()>;
}