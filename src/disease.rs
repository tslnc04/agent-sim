pub trait Disease {
    fn will_infect(&self) -> bool;
    fn mutate(&self) -> Self
    where
        Self: Sized;
}
