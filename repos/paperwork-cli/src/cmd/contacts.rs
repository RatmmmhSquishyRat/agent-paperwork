//! `paperwork contacts` — list all registered agents.

use anyhow::Result;
use clap::Args;
use serde::Serialize;

use crate::cmd::Context;
use crate::output::{self, OutputMode};

/// List all registered agents
#[derive(Args)]
pub struct ContactsArgs {}

#[derive(Serialize)]
struct ContactJson {
    agent: String,
    profile: String,
}

pub fn run(ctx: &Context, _args: ContactsArgs) -> Result<()> {
    let contacts = paperwork_core::ops::contacts::contacts_list(&ctx.root)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match ctx.mode {
        OutputMode::Json => {
            let items: Vec<ContactJson> = contacts
                .iter()
                .map(|c| ContactJson {
                    agent: c.agent.clone(),
                    profile: c.profile_path.clone(),
                })
                .collect();
            output::print_json(&items);
        }
        OutputMode::Plain => {
            let mut out = String::from("| Agent | Profile |\n|-------|--------|\n");
            for c in &contacts {
                out.push_str(&format!("| {} | {} |\n", c.agent, c.profile_path));
            }
            output::print_plain(&out);
        }
        OutputMode::Default => {
            if contacts.is_empty() {
                output::print_default("No contacts registered.");
            } else {
                let mut out = String::from("AGENT   PROFILE\n");
                for c in &contacts {
                    out.push_str(&format!("{:<8}{}\n", c.agent, c.profile_path));
                }
                output::print_default(out.trim_end());
            }
        }
    }

    Ok(())
}
