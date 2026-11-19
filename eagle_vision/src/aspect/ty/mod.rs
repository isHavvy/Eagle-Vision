use bevy::prelude::*;
use crate::eaglevision::GuaranteedMarkerComponent;
use crate::aspect::text::Text;
use crate::private::EagleVisionSealed;

/// An entity with this component is a type.
#[derive(Debug, Component)]
pub struct Ty;

/// An entity with this component has the type this component points to.
#[derive(Debug, Component, PartialEq, Eq)]
pub struct ActualTy(pub(crate) Entity);

impl FromWorld for ActualTy {
    fn from_world(world: &mut World) -> Self {
        Self(world.query_filtered::<Entity, With<TyUnknown>>().single(&world).unwrap())
    }
}

pub enum CoreTy {
    Unknown,
    Unit,
    Boolean,
    Int32,
    Float64,

    // #TODO(Havvy): Make this generic somehow OR give every function its own ty just like Rust does?
    Function,

    // #TODO(Havvy): Make this not be a core ty.
    Utf8,
}

#[derive(Debug, Default, Component)]
pub struct TyUnknown;

impl EagleVisionSealed for TyUnknown {}
impl GuaranteedMarkerComponent for TyUnknown {}

#[derive(Debug, Default, Component)]
pub struct TyUnit;

impl EagleVisionSealed for TyUnit {}
impl GuaranteedMarkerComponent for TyUnit {}

#[derive(Debug, Default, Component)]
pub struct TyInt32;

impl EagleVisionSealed for TyInt32 {}
impl GuaranteedMarkerComponent for TyInt32 {}

#[derive(Debug, Default, Component)]
pub struct TyFloat64;

impl EagleVisionSealed for TyFloat64 {}
impl GuaranteedMarkerComponent for TyFloat64 {}

#[derive(Debug, Default, Component)]
pub struct TyBoolean;

impl EagleVisionSealed for TyBoolean {}
impl GuaranteedMarkerComponent for TyBoolean {}

#[derive(Debug, Default, Component)]
pub struct TyUtf8;

impl EagleVisionSealed for TyUtf8 {}
impl GuaranteedMarkerComponent for TyUtf8 {}

#[derive(Debug, Default, Component)]
pub struct TyFunction;

impl EagleVisionSealed for TyFunction {}
impl GuaranteedMarkerComponent for TyFunction {}