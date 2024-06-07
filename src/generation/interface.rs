/// A trait for transpiling into a GeneratedTypescript instance.
pub trait RoswaalTypescriptGenerate<Typescript> {
    /// The associated typescript code for this test command.
    fn typescript(&self) -> Typescript;
}
