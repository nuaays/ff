use std::sync::Arc;

use async_trait::async_trait;
use ff_api::SafeStreamEx;
use futures::AsyncWriteExt;
use smol::net::TcpStream;

use crate::packet::Action;

#[derive(Debug, Clone, Copy)]
pub enum State<T> {
    Next,
    Release,
    Accept(T),
}

#[async_trait]
pub trait Handler<D, C, A> {
    async fn dispose(&self, o: D, c: C) -> ff_api::Result<State<A>>;
}

#[async_trait]
pub trait Dispatch<H, C> {
    async fn dispatch(self, h: H, cx: C) -> ff_api::Result<()>;
}

#[async_trait]
pub trait StrategyEx<R, S, C> {
    async fn select(self, strategys: S, cx: C) -> ff_api::Result<R>;
}

// pub type TcpStreamRollback = Rollback<TcpStream, Buffer<u8>>;
pub type SafeTcpStream = ff_api::SafeStream<TcpStream>;

pub type DynHandler<C, A> = dyn Handler<SafeTcpStream, C, A> + Send + Sync + 'static;

#[async_trait]
impl<C> Dispatch<Arc<Vec<Arc<Box<DynHandler<C, ()>>>>>, C> for TcpStream
where
    C: Clone + Send + Sync + 'static,
{
    #[inline]
    async fn dispatch(
        self,
        handlers: Arc<Vec<Arc<Box<DynHandler<C, ()>>>>>,
        cx: C,
    ) -> ff_api::Result<()> {
        let mut io = self.as_safe_stream();
        for handle in handlers.iter() {
            let handle = handle.clone();

            match handle.dispose(io.clone(), cx.clone()).await? {
                State::Accept(()) => return Ok(()),
                State::Release => return Ok(()),
                State::Next => {
                    log::debug!("[dispatch] Next handler, rollback");
                }
            }
        }

        log::warn!(
            "Can't find a suitable processor {}",
            io.peer_addr().unwrap()
        );

        let _ = io.close().await;

        Err(ff_api::ErrorKind::UnHandler.into())
    }
}

#[async_trait]
impl<C> StrategyEx<Action, Arc<Vec<Arc<Box<DynHandler<C, Action>>>>>, C>
    for ff_api::SafeStream<TcpStream>
where
    C: Clone + Send + Sync + 'static,
{
    #[inline]
    async fn select(
        self,
        strategys: Arc<Vec<Arc<Box<DynHandler<C, Action>>>>>,
        cx: C,
    ) -> ff_api::Result<Action> {
        // let mut io = self.roll();
        let mut io = self;

        // io.begin().await?;

        for handle in strategys.iter() {
            let handle = handle.clone();

            match handle.dispose(io.clone(), cx.clone()).await? {
                State::Accept(action) => {
                    return Ok(action);
                }
                State::Release => return Ok(Action::Nothing),
                State::Next => {
                    // io.back().await?;
                    log::debug!("[dispatch] Next handler, rollback");
                }
            }
        }

        log::warn!(
            "Can't find a suitable processor {}",
            io.peer_addr().unwrap()
        );

        let _ = io.close().await;

        Err(ff_api::ErrorKind::UnHandler.into())
    }
}
