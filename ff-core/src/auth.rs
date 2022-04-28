use async_trait::async_trait;
use ff_api::FFAuth;
use futures::{AsyncRead, AsyncWrite};

#[allow(unused)]
pub struct TokenAuth {
    token: String,
}

impl TokenAuth {
    pub fn new(token: &str) -> Self {
        Self {
            token: token.into(),
        }
    }
}

#[async_trait]
impl<T> FFAuth<T> for TokenAuth
where
    T: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
{
    async fn auth(&self, _: &mut T) -> ff_api::Result<()> {
        Ok(())
    }
}
