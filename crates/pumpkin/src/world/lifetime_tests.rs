use crate::{data::VanillaData, server::Server};
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration};
use pumpkin_data::dimension::Dimension;
use std::sync::{Arc, RwLock};

#[tokio::test]
async fn unloaded_worlds_are_released() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let basic = BasicConfiguration {
        default_level_name: directory.path().to_string_lossy().into_owned(),
        allow_nether: false,
        allow_end: false,
        use_favicon: false,
        ..Default::default()
    };
    let mut advanced = AdvancedConfiguration::default();
    advanced.networking.bedrock.online_mode = false;
    let data = VanillaData {
        banned_ip_list: RwLock::default(),
        banned_player_list: RwLock::default(),
        operator_config: RwLock::default(),
        user_cache: RwLock::default(),
        whitelist_config: RwLock::default(),
    };
    let server = Server::new(basic, advanced, data).await;
    let mut unloaded = Vec::new();
    for id in 0..4 {
        let world = server.create_world(format!("temporary_{id}"), Dimension::OVERWORLD);
        unloaded.push((Arc::downgrade(&world), Arc::downgrade(&world.level)));
        server.unload_world(world.get_world_name()).await?;
        drop(world);
    }
    assert_eq!(server.worlds.load().len(), 1);
    server.shutdown().await;
    assert!(
        unloaded
            .iter()
            .all(|(world, level)| world.upgrade().is_none() && level.upgrade().is_none()),
        "unloaded worlds or their levels remain allocated"
    );
    Ok(())
}
