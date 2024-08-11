use std::io::Cursor;

use crate::args;
use super::{ResultRun, ResultStream, Error};
use tokio_util::io::ReaderStream;

#[inline]
fn err_into<E: std::error::Error>(e: E) -> Error {
    Error::Custom(std::borrow::Cow::Owned(e.to_string()))
}
#[inline]
fn res_into<T,E: std::error::Error>(r: Result<T, E>) -> Result<T, Error> {
    r.map_err(err_into)
}

pub async fn run(_: crate::config::Config, args: args::FromContentArgs, input: &str) -> ResultRun {
    use tokio_stream::StreamExt;
    if args.file {
        let file = tokio::fs::File::open(&input).await.map_err(err_into)?;
        let stream = ReaderStream::new(file).map(|r| -> ResultStream {
            let bytes = res_into(r)?;
            String::from_utf8(bytes.as_ref().to_vec()).map_err(err_into)
        });
        Ok(Box::pin(stream))
    } else {
        let stream = ReaderStream::new(Cursor::new(String::from(input).into_bytes()))
            .map(res_into)
            .map(|r| 
                r.and_then(|v| 
                    res_into(std::str::from_utf8(v.as_ref())).map(String::from)
                )
                
            );
        Ok(Box::pin(stream))
    }
}