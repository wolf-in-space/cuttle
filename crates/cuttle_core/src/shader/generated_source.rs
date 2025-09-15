use bevy_asset::io::PathStream;
use bevy_asset::io::{AssetReader, AssetReaderError, AsyncSeekForward, Reader, StackFuture};
use futures_io::AsyncRead;
use std::io::Result as IoResult;
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

struct GeneratedAssetReader;
impl AssetReader for GeneratedAssetReader {
    async fn read<'a>(&'a self, _path: &'a Path) -> Result<EmptyReader, AssetReaderError> {
        Ok(EmptyReader)
    }

    async fn read_meta<'a>(&'a self, _path: &'a Path) -> Result<EmptyReader, AssetReaderError> {
        Ok(EmptyReader)
    }

    async fn read_directory<'a>(
        &'a self,
        _path: &'a Path,
    ) -> Result<Box<PathStream>, AssetReaderError> {
        unimplemented!()
    }

    async fn is_directory<'a>(&'a self, _path: &'a Path) -> Result<bool, AssetReaderError> {
        Ok(false)
    }
}

struct EmptyReader;
impl AsyncRead for EmptyReader {
    fn poll_read(self: Pin<&mut Self>, _: &mut Context<'_>, _: &mut [u8]) -> Poll<IoResult<usize>> {
        Poll::Ready(Ok(0))
    }
}

impl AsyncSeekForward for EmptyReader {
    fn poll_seek_forward(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        _: u64,
    ) -> Poll<futures_io::Result<u64>> {
        Poll::Ready(Ok(0))
    }
}

impl Reader for EmptyReader {
    fn read_to_end<'a>(
        &'a mut self,
        _: &'a mut Vec<u8>,
    ) -> StackFuture<'a, std::io::Result<usize>, 80> {
        StackFuture::from(async move { Ok(0) })
    }
}
