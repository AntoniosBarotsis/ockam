use anyhow::anyhow;
use clap::Args;

use ockam::{Context, TcpTransport};
use ockam_api::cloud::{project::CreateProject, MessagingClient};
use ockam_multiaddr::MultiAddr;

use crate::old::identity::load_or_create_identity;
use crate::util::{embedded_node, DEFAULT_CLOUD_ADDRESS};
use crate::IdentityOpts;

#[derive(Clone, Debug, Args)]
pub struct CreateCommand {
    /// Id of the space the project belongs to.
    #[clap(display_order = 1001)]
    space_id: String,

    /// Name of the project.
    #[clap(display_order = 1002)]
    project_name: String,

    /// Services enabled for this project.
    #[clap(display_order = 1003)]
    services: Vec<String>,

    /// Ockam's cloud address. Argument used for testing purposes.
    #[clap(hide = true, last = true, display_order = 1100, default_value = DEFAULT_CLOUD_ADDRESS)]
    addr: MultiAddr,

    #[clap(flatten)]
    identity_opts: IdentityOpts,
}

impl CreateCommand {
    pub fn run(cmd: CreateCommand) {
        embedded_node(create, cmd);
    }
}

async fn create(mut ctx: Context, cmd: CreateCommand) -> anyhow::Result<()> {
    let _tcp = TcpTransport::create(&ctx).await?;

    // TODO: The identity below will be used to create a secure channel when cloud nodes support it.
    let identity = load_or_create_identity(&ctx, cmd.identity_opts.overwrite).await?;

    let route = ockam_api::multiaddr_to_route(&cmd.addr)
        .ok_or_else(|| anyhow!("failed to parse address"))?;
    let mut api = MessagingClient::new(route, identity, &ctx).await?;
    let request = CreateProject::new(cmd.project_name, &cmd.services);
    let res = api.create_project(&cmd.space_id, request).await?;
    println!("{res:#?}");

    ctx.stop().await?;
    Ok(())
}
