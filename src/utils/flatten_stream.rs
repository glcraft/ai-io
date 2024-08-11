use tokio_stream::Stream;

pub trait FlattenTrait: Stream
{
    fn flatten_iter(self) -> Flatten<StreamIter<Self, <Self as Stream>::Item>>
    where 
        Self: Sized,
        Self::Item: Iterator
    {
        Flatten {
            stream: StreamIter(self),
            current: None
        }
    }
    fn flatten_result_iter<It, E>(self) -> Flatten<StreamResultIter<Self, It, E>>
    where
        Self: Sized + Stream<Item=Result<It, E>>,
        It: Iterator 
    {
        Flatten {
            stream: StreamResultIter(self),
            current: None
        }
    }
}

#[pin_project::pin_project]
pub struct Flatten<St> 
where
    St: Stream,
{
    #[pin]
    stream: St,
    current: Option<<St as Stream>::Item>
}
#[pin_project::pin_project]
pub struct StreamIter<St, It>(#[pin] St)
where
    St: Stream<Item=It>,
    It: Iterator;

impl<St, It> Stream for StreamIter<St, It>
where
    St: Stream<Item=It>,
    It: Iterator
{
    type Item = St::Item;
    #[inline]
    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<<Self as Stream>::Item>> {
        self.as_mut().project().0.poll_next(cx)
    }
}

#[pin_project::pin_project]
pub struct StreamResultIter<St, It, E>(#[pin] St)
where
    St: Stream<Item=Result<It, E>>,
    It: Iterator;

impl<St, It, E> Stream for StreamResultIter<St, It, E>
where
    St: Stream<Item=Result<It, E>>,
    It: Iterator
{
    type Item = St::Item;
    #[inline]
    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<<Self as Stream>::Item>> {
        self.as_mut().project().0.poll_next(cx)
    }
}

impl<St: ?Sized> FlattenTrait for St where St: Stream {}

/// Implementation of Flatten for Result of Iterator
impl<St, It, E> Stream for Flatten<StreamResultIter<St, It, E>>
where
    St: Stream<Item=Result<It, E>>,
    It: Iterator,
{
    type Item = Result<It::Item, E>;
    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<<Self as Stream>::Item>> {
        use std::task::Poll::*;
        let self_project = self.as_mut().project();
        match self_project.current {
            current @ Some(Ok(_)) => {
                let item = unsafe { current.as_mut().unwrap_unchecked().as_mut().unwrap_unchecked().next() };
                match item {
                    None => {
                        *current = None;
                        self.poll_next(cx)
                    }
                    Some(v) => Ready(Some(Ok(v))),
                }
            }
            e @ Some(Err(_)) => {
                let mut emv = None;
                std::mem::swap(&mut emv, e);
                let Some(Err(emv)) = emv else { unreachable!() };
                Ready(Some(Err(emv)))
            },
            None => {
                let item = self_project.stream.poll_next(cx);
                match item {
                    Ready(Some(item)) => {
                        *self_project.current = Some(item);
                        self.poll_next(cx)
                    }
                    Ready(None) => Ready(None),
                    Pending => Pending,
                }
            }
        }
    }
}

// Implementation of Flatten for Iterator

impl<St, It> Stream for Flatten<StreamIter<St, It>>
where
    St: Stream<Item = It>,
    It: Iterator
{
    type Item = <It as Iterator>::Item;
    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<<Self as Stream>::Item>> {
        use std::task::Poll::*;
        let self_project = self.as_mut().project();
        match self_project.current {
            current @ Some(_) => {
                let item = unsafe { current.as_mut().unwrap_unchecked().next()};
                match item {
                    None => {
                        *current = None;
                        self.poll_next(cx)
                    }
                    v => Ready(v),
                }
            }
            current @ None => {
                let item = self_project.stream.poll_next(cx);
                match item {
                    Ready(Some(item)) => {
                        *current = Some(item);
                        self.poll_next(cx)
                    }
                    Ready(None) => Ready(None),
                    Pending => Pending,
                }
            }
        }
    }
}