pub mod entities;

pub trait TryCollectExt<I, T, E>: Iterator<Item = I> + Sized
where
    I: Into<Result<T, E>>,
{
    fn try_collect<B: FromIterator<T>>(self) -> Result<B, E> {
        let mut error = None;

        let collected: B = self
            .map_while(|item| match item.into() {
                Ok(v) => Some(v),
                Err(err) => {
                    error = Some(err);
                    None
                }
            })
            .collect();

        match error {
            Some(err) => Err(err),
            None => Ok(collected),
        }
    }
}

impl<S, I, T, E> TryCollectExt<I, T, E> for S
where
    S: Iterator<Item = I>,
    I: Into<Result<T, E>>,
{
}
