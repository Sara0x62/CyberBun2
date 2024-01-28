use poise::{serenity_prelude::{Context, FullEvent}, FrameworkContext};
use super::{Data, Error};


pub async fn event_handler (
    ctx: &Context,
    event: &FullEvent,
    _framework: FrameworkContext<'_, Data, Error>,
    data: &Data) -> Result<(), Error>
{
    Ok(())
}