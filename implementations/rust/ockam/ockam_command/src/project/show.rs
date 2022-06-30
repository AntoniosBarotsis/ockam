use anyhow::anyhow;
use clap::Args;

use ockam::{Context, TcpTransport};
use ockam_api::cloud::MessagingClient;
use ockam_multiaddr::MultiAddr;

use crate::old::identity::load_or_create_identity;
use crate::util::{embedded_node, DEFAULT_CLOUD_ADDRESS};
use crate::IdentityOpts;

#[derive(Clone, Debug, Args)]
pub struct ShowCommand {
    /// Id of the space.
    #[clap(display_order = 1001)]
    space_id: String,

    /// Id of the project.
    #[clap(display_order = 1002)]
    project_id: String,

    /// Ockam's cloud address. Argument used for testing purposes.
    #[clap(hide = true, display_order = 1100, default_value = DEFAULT_CLOUD_ADDRESS)]
    addr: MultiAddr,

    #[clap(flatten)]
    identity_opts: IdentityOpts,
}

impl ShowCommand {
    pub fn run(cmd: ShowCommand) {
        embedded_node(show, cmd);
    }
}

async fn show(mut ctx: Context, cmd: ShowCommand) -> anyhow::Result<()> {
    let _tcp = TcpTransport::create(&ctx).await?;

    // TODO: The identity below will be used to create a secure channel when cloud nodes support it.
    let identity = load_or_create_identity(&ctx, cmd.identity_opts.overwrite).await?;

    let route = ockam_api::multiaddr_to_route(&cmd.addr)
        .ok_or_else(|| anyhow!("failed to parse address"))?;
    let mut api = MessagingClient::new(route, identity, &ctx).await?;
    let res = api.get_project(&cmd.space_id, &cmd.project_id).await?;
    println!("{res:#?}");

    ctx.stop().await?;
    Ok(())
}
