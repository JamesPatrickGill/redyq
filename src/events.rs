use strum_macros::EnumString;

#[derive(EnumString)]
pub enum QueueEvent {
    AddWaiting,
}
