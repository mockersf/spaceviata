use std::io::{BufRead, BufReader};

use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};

#[derive(Debug, TypeUuid)]
#[uuid = "88608822-C4D7-4A30-A4D8-77F00412FE91"]
pub struct Names {
    pub names: Vec<String>,
}

#[derive(Default)]
pub struct NamesLoader;

impl AssetLoader for NamesLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            load_context.set_default_asset(LoadedAsset::new(
                BufReader::new(bytes)
                    .lines()
                    .collect::<Result<Vec<String>, _>>()
                    .map(|v| Names { names: v })?,
            ));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["names"]
    }
}
