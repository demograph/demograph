use core::fmt;
use futures::future::Future;
use futures::Async;
use futures::AsyncSink;
use futures::Sink;

/// Future for the `flatten_sink` combinator, flattening a
/// future-of-a-sink to get just the result of the final sink as a sink.
///
/// This is created by the `Future::flatten_sink` method.
#[must_use = "sinks do nothing unless polled"]
pub struct FlattenSink<F>
where
    F: Future,
    <F as Future>::Item: Sink<SinkError = F::Error>,
{
    state: State<F>,
}

impl<F> fmt::Debug for FlattenSink<F>
where
    F: Future + fmt::Debug,
    <F as Future>::Item: Sink<SinkError = F::Error> + fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("FlattenSink")
            .field("state", &self.state)
            .finish()
    }
}

#[derive(Debug)]
enum State<F>
where
    F: Future,
    <F as Future>::Item: Sink<SinkError = F::Error>,
{
    // future is not yet called or called and not ready
    Future(F),
    // future resolved to Sink
    Sink(F::Item),
    // EOF after future resolved to error
    Eof,
    // after EOF after future resolved to error
    Done,
}

pub fn new<F>(f: F) -> FlattenSink<F>
where
    F: Future,
    <F as Future>::Item: Sink<SinkError = F::Error>,
{
    FlattenSink {
        state: State::Future(f),
    }
}

impl<F: Future> FlattenSink<F>
where
    <F as Future>::Item: Sink<SinkError = F::Error>,
{
    fn handle_sink<SF>(&mut self, mut f: SF) -> Result<Async<()>, F::Error>
    where
        SF: FnMut(
            &mut dyn Sink<
                SinkItem = <<F as Future>::Item as Sink>::SinkItem,
                SinkError = <<F as Future>::Item as Sink>::SinkError,
            >,
        ) -> Result<Async<()>, F::Error>,
    {
        loop {
            let (next_state, ret_opt) = match self.state {
                State::Future(ref mut f) => {
                    match f.poll() {
                        Ok(Async::NotReady) => {
                            // State is not changed, early return.
                            return Ok(Async::NotReady);
                        }
                        Ok(Async::Ready(sink)) => {
                            // Future resolved to sink.
                            // We do not return, but initiate dispatch that
                            // sink in the next loop iteration.
                            (State::Sink(sink), None)
                        }
                        Err(e) => (State::Eof, Some(Err(e))),
                    }
                }
                State::Sink(ref mut s) => {
                    // Just forward call to the sink,
                    // do not track its state.
                    return f(s);
                }
                State::Eof => (State::Done, Some(Ok(Async::Ready(())))),
                State::Done => {
                    panic!("poll called after eof");
                }
            };

            self.state = next_state;
            if let Some(ret) = ret_opt {
                return ret;
            }
        }
    }
}

impl<F> Sink for FlattenSink<F>
where
    F: Future,
    <F as Future>::Item: Sink<SinkError = F::Error>,
{
    type SinkItem = <F::Item as Sink>::SinkItem;
    type SinkError = <F::Item as Sink>::SinkError;

    fn start_send(
        &mut self,
        item: Self::SinkItem,
    ) -> Result<AsyncSink<Self::SinkItem>, Self::SinkError> {
        loop {
            let (next_state, ret_opt) = match self.state {
                State::Future(ref mut f) => {
                    match f.poll() {
                        Ok(Async::NotReady) => {
                            // State is not changed, early return.
                            return Ok(AsyncSink::NotReady(item));
                        }
                        Ok(Async::Ready(sink)) => {
                            // Future resolved to sink.
                            // We do not return, but initiate dispatch that
                            // sink in the next loop iteration.
                            (State::Sink(sink), None)
                        }
                        Err(e) => (State::Eof, Some(Err(e))),
                    }
                }
                State::Sink(ref mut s) => {
                    // Just forward call to the sink,
                    // do not track its state.
                    return s.start_send(item);
                }
                State::Eof => (State::Done, Some(Ok(AsyncSink::Ready))),
                State::Done => {
                    panic!("start_send called after eof");
                }
            };

            self.state = next_state;
            if let Some(ret) = ret_opt {
                return ret;
            }
        }
        //        return self.handle_sink(move |sink| sink.start_send(item));
    }

    fn poll_complete(&mut self) -> Result<Async<()>, Self::SinkError> {
        //        loop {
        //            let (next_state, ret_opt) = match self.state {
        //                State::Future(ref mut f) => {
        //                    match f.poll() {
        //                        Ok(Async::NotReady) => {
        //                            // State is not changed, early return.
        //                            return Ok(Async::NotReady);
        //                        }
        //                        Ok(Async::Ready(sink)) => {
        //                            // Future resolved to sink.
        //                            // We do not return, but initiate dispatch that
        //                            // sink in the next loop iteration.
        //                            (State::Sink(sink), None)
        //                        }
        //                        Err(e) => (State::Eof, Some(Err(e))),
        //                    }
        //                }
        //                State::Sink(ref mut s) => {
        //                    // Just forward call to the sink,
        //                    // do not track its state.
        //                    return s.poll_complete();
        //                }
        //                State::Eof => (State::Done, Some(Ok(Async::Ready(None)))),
        //                State::Done => {
        //                    panic!("poll called after eof");
        //                }
        //            };
        //
        //            self.state = next_state;
        //            if let Some(ret) = ret_opt {
        //                return ret;
        //            }
        //        }
        return self.handle_sink(|sink| sink.poll_complete());
    }

    fn close(&mut self) -> Result<Async<()>, Self::SinkError> {
        //        loop {
        //            let (next_state, ret_opt) = match self.state {
        //                State::Future(ref mut f) => {
        //                    match f.poll() {
        //                        Ok(Async::NotReady) => {
        //                            // State is not changed, early return.
        //                            return Ok(Async::NotReady);
        //                        }
        //                        Ok(Async::Ready(sink)) => {
        //                            // Future resolved to sink.
        //                            // We do not return, but initiate dispatch that
        //                            // sink in the next loop iteration.
        //                            (State::Sink(sink), None)
        //                        }
        //                        Err(e) => (State::Eof, Some(Err(e))),
        //                    }
        //                }
        //                State::Sink(ref mut s) => {
        //                    // Just forward call to the sink,
        //                    // do not track its state.
        //                    return s.close();
        //                }
        //                State::Eof => (State::Done, Some(Ok(Async::Ready(None)))),
        //                State::Done => {
        //                    panic!("poll called after eof");
        //                }
        //            };
        //
        //            self.state = next_state;
        //            if let Some(ret) = ret_opt {
        //                return ret;
        //            }
        //        }
        return self.handle_sink(|sink| sink.close());
    }
}

pub trait FlattenSinkOps {
    fn flatten_sink(self) -> FlattenSink<Self>
    where
        Self: Future,
        <Self as Future>::Item: Sink<SinkError = <Self as Future>::Error>,
        Self: Sized,
    {
        new(self)
    }
}

impl<F: Future> FlattenSinkOps for F where <F as Future>::Item: Sink<SinkError = F::Error> {}
