use bevy::ecs::schedule::{ScheduleLabel, SystemSet};

/// The schedule that runs once when the language engine is started.
#[derive(ScheduleLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Startup;

/// The schedule that runs once directly after [`Startup`].
#[derive(ScheduleLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PostStartup;

/// The schedule that drives the engine.
#[derive(ScheduleLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Update;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EngineSet {
    ExecuteOps,
    UpdateSubscriptions,
}