//! The Eagle Vision Source controller
//! 
//! God object that actually holds the source and maintains its invariants.

// #[TODO(Havvy)]: Don't return bevy Entities. Instead return wrappers that guarantee more information.

use std::cell::RefCell;
use std::num::NonZeroU16;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::TryRecvError;

use bevy::app::App;
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::ecs::system::SystemId;
use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use eagle_squak::aspect::AspectKind;
use eagle_squak::id::AspectedId;
use eagle_squak::id_allocator::SourceIdAllocator;
use eagle_squak::protocol::{ClientHello, HandshakeError, ServerWelcome};

use crate::bytecode::chunk::Chunk;
use crate::bytecode::vm::{InterpretError, VirtualMachine};
use crate::diagnostic::{Diagnostic, Diagnostics, DiagnosticsOf};
use crate::aspect;
use crate::aspect::exe::Executable;
use crate::aspect::expression::{BlockExpression, CallExpression, CoreLiteralValue, Expression, ExpressionKind, LiteralExpression, MathOperator, MathOperatorExpression};
use crate::aspect::function::Function;
use crate::aspect::locale::Locale;
use crate::aspect::metadata::ProjectName;
use crate::aspect::text::Text;
use crate::aspect::ty::{ActualTy, CoreTy, TyBoolean, TyFloat64, TyFunction, TyInt32, TyUnit, TyUnknown, TyUtf8};
use crate::init::{EagleVisionInitialSettings, EagleVisionInitializationPlugin};
use crate::private::EagleVisionSealed;
use crate::server::{EagleVisionServerPlugin, EagleVisionUpdate, handle_hello};

#[derive(Debug)]
pub struct EagleVision {
    app: App,
}

impl EagleVision {
    pub fn new_project(settings: EagleVisionInitialSettings, init_logging: bool) -> Self {
        let mut app = App::empty();

        if init_logging {
            app.add_plugins((
                LogPlugin {
                    level: Level::DEBUG,   
                    ..Default::default()
                },
            ));
        }
        app.add_plugins((
            EagleVisionCorePlugin,
            EagleVisionInitializationPlugin { settings },
            EagleVisionServerPlugin,
        ));

        app.finish();
        app.cleanup();

        let world = app.world_mut();
        world.run_schedule(Startup);
        world.flush();
        world.clear_trackers();

        Self { app }
    }

    pub fn new_project_with_single_client(
        settings: EagleVisionInitialSettings,
        hello: ClientHello,
        init_logging: bool
    ) -> Result<(Self, ServerWelcome), HandshakeError> {
        let mut ev = Self::new_project(settings, init_logging);

        ev.app.add_plugins(crate::server::single::SingleClientPlugin);
        ev.app.finish();
        ev.app.cleanup();

        let welcome = ev.handle_hello(hello)?;

        Ok((ev, welcome))
    }

    // pub fn run(mut self) {
    //     loop {
    //         let world = self.app.world_mut();
            
    //         loop {
    //             match self.inbound.try_recv() {
    //                 Ok(cmd) => self.pending.push_back(cmd),
    //                 Err(TryRecvError::Empty) => break,
    //                 Err(TryRecvError::Disconnected) => return,
    //             }
    //         }

    //         world.run_schedule(EagleVisionUpdate);
    //         world.flush();
    //         world.clear_trackers();

    //         if !world.resource::<WorkPending>().is_set() {
    //             // Idle: sleep until a command arrives.
    //             match self.inbound.recv() {
    //                 Ok(cmd) => self.pending.push_back(cmd),
    //                 Err(_) => return,
    //             }
    //         }
    //     }
    // }

    /// Execute one turn of the language server.
    /// 
    /// This will read all client requests and send out subscription updates
    /// and other server messages.
    /// 
    /// Long running tasks will make progress and may or may notcomplete.
    pub fn turn(&mut self) {
        let world = self.app.world_mut();
        world.resource::<WorkPending>().clear();
        world.run_schedule(EagleVisionUpdate);
        world.flush();
        world.clear_trackers();
    }

    pub(crate) fn handle_hello(&mut self, hello: ClientHello) -> Result<ServerWelcome, HandshakeError> {
        handle_hello(self.app.world_mut(), hello)
    }

    #[cfg(test)]
    pub fn world(&self) -> &World { self.app.world() }

    #[cfg(test)]
    pub fn world_mut(&mut self) -> &mut World { self.app.world_mut() }
}

/// The [`EagleVisionId`](eagle_squak::id::EagleVisionId) generator for the server.
#[derive(Resource)]
pub struct ServerIdGen(pub SourceIdAllocator);

impl ServerIdGen {
    // We'll probably want the source code file IDs for this project to be 0.
    // Could make this 1, but then all dependencies will probably be at 3 or higher.
    // Making this 0xFFFF makes it 2 or higher, but if clients are allocated going
    // down, then dependencies could have sources 1 through however many is needed.
    //
    // No matter what, we'll need a resource that keeps track of which source id
    // is which source once we start implementing dependencies or multiple clients.
    const SOURCE_LANGUAGE_SERVER: u16 = -1i16 as u16;

    fn new() -> Self {
        Self(SourceIdAllocator::new(Self::SOURCE_LANGUAGE_SERVER))
    }
}

impl Default for ServerIdGen {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for ServerIdGen {
    type Target = SourceIdAllocator;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Resource)]
pub struct SourceAllocator(NonZeroU16);

impl SourceAllocator {
    pub fn new() -> Self {
        // 0 is reserved for the project source file.
        // 1 is reserved for the core library.
        // Thus we start at 2.
        Self(2.try_into().unwrap())
    }

    pub fn next(&mut self) -> Option<NonZeroU16> {
        // 0xFFFF is reserved for the EagleVision server.
        if self.0.get() == 0xFFFF {
            None
        } else {
            let ret = Some(self.0);
            self.0.saturating_add(1);
            ret
        }
    }
}

impl Default for SourceAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Resource, Default)]
pub struct WorkPending(pub AtomicBool);

impl WorkPending {
    fn set(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    fn clear(&self) {
        self.0.store(false, Ordering::Relaxed);
    }

    fn is_set(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

pub struct EagleVisionCorePlugin;

impl Plugin for EagleVisionCorePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ServerIdGen>()
            .init_resource::<WorkPending>()
            .init_resource::<SourceAllocator>()
            .init_schedule(Startup)
        ;
    }
}

// /// Metadata methods
// impl EagleVision {
//     pub fn get_project_name(&self, locale: Entity) -> Option<&str> {
//         let name = self.get_project_name_text();
//         let text = self.app.world().get_entity(name).unwrap().get::<Text>().unwrap();
//         text.get_value(locale)
//     }

//     pub fn get_project_name_text(&self) -> Entity {
//         self.app.world().get_resource::<ProjectName>().unwrap().name()
//     }
// }

// /// Locale methods
// impl EagleVision {
//     pub fn get_locale(&self, code: &str) -> Option<Entity> {
//         let mut locale_query = self.app.world().try_query::<(Entity, &Locale)>().unwrap();

//         locale_query.iter(&self.app.world())
//         .find(|&(_entity, locale)| locale.code() == code)
//         .map(|(entity, _locale)| entity)
//     }

//     pub fn insert_locale(&mut self, locale: String) -> Entity {
//         InsertLocaleSystem::run(&mut self.app.world_mut(), locale)
//     }

//     pub fn update_locale(&mut self, locale: Entity, code: String) {
//         UpdateLocaleSystem::run(&mut self.app.world_mut(), (locale, code))
//     }
// }

// /// Text methods
// impl EagleVision {
//     pub fn text_value(&self, text: Entity, locale: Entity) -> Option<&str> {
//         self.app.world().entity(text).get::<Text>().unwrap().get_value(locale)
//     }

//     pub fn upsert_text(&mut self, text: Entity, locale: Entity, value: impl Into<String>) {
//         self.app.world_mut().entity_mut(text).get_mut::<Text>().unwrap().upsert_value(locale, value.into());
//     }
// }

// /// Diagnostics methods
// impl EagleVision {
//     pub fn diagnostics_of(&self, entity: Entity) -> DiagnosticsOf {
//         let entity_ref = self.app.world().entity(entity);
//         let diagnostics = entity_ref.get::<Diagnostics>()
//             .map(|d| d.iter())
//             .unwrap_or_default()
//             .map(|&entity| self.app.world().entity(entity).get::<Diagnostic>().unwrap())
//             .cloned()
//             .collect();

//         DiagnosticsOf(diagnostics)
//     }
// }

// /// Executable methods
// impl EagleVision {
//     pub fn new_executable(&mut self) -> Entity {
//         let unit = TyUnit::get(self);

//         let expression = Expression {
//             kind: ExpressionKind::Block,
//             expected_ty: None,
//         };

//         let block = self.app.world_mut().spawn((
//             expression,
//             BlockExpression::new(),
//             ActualTy(unit)
//         )).id();

//         self.app.world_mut().spawn(Executable {
//             main_block: block
//         }).id()
//     }

//     // pub fn executable(&self, executable: Entity) -> EagleVisionExecutable<'_, SharedStorage<'_>> {
//     //     EagleVisionExecutable::new(self, executable)
//     // }

//     // pub fn executable_mut(&mut self, executable: Entity) -> EagleVisionExecutable<'_, MutableStorage<'_>> {
//     //     EagleVisionExecutable::new_mut(self, executable)
//     // }

//     pub fn execute_chunk(&self, chunk: Entity) -> Result<String, InterpretError> {
//         let chunk = self.app.world().entity(chunk).get::<Chunk>().unwrap();
//         let full_output = Rc::new(RefCell::new(String::new()));
//         {
//             let vm_full_output = Rc::clone(&full_output);
//             let mut vm = VirtualMachine::new(
//                 chunk.clone(),
//                 Box::new(move |output| { *vm_full_output.borrow_mut() += &output; })
//             );
//             vm.interpret()?;
//         }
        
//         // UNWRAP: The only other Rc pointer is collected when the virtual machine is dropped
//         //         in the previous block.
//         Ok(Rc::try_unwrap(full_output).unwrap().into_inner())
//     }
// }

// /// Expression methods
// impl EagleVision {
//     pub fn new_literal_expression(&mut self, value: CoreLiteralValue) -> Entity {
//         LiteralExpression::spawn(self, value)
//     }

//     pub fn new_operator_expression(&mut self, op: MathOperator, lhs: Entity, rhs: Entity, expected_ty: Option<Entity>) -> Entity {
//         MathOperatorExpression::spawn(self, op, lhs, rhs, expected_ty)
//     }

//     pub fn new_print_expression(&mut self, printed_expression: Entity) -> Entity {
//         crate::aspect::expression::spawn_print_expression( self, printed_expression)
//     }

//     pub fn push_block_expression_statement(&mut self, block_expression: Entity, statement: Entity) {
//         crate::aspect::expression::push_block_expression_statement(self.app.world_mut(), block_expression, statement);
//     }

//     pub fn new_call_expression(&mut self, call_expression: CallExpression) -> Entity {
//         CallExpression::spawn(self, call_expression)
//     }

//     pub fn expression(&self, expression: Entity) -> EagleVisionExpression {
//         EagleVisionExpression::new(self, expression)
//     }

//     pub fn expression_mut(&mut self, expression: Entity) -> EagleVisionExpressionMut {
//         EagleVisionExpressionMut::new(self, expression)
//     }
// }

// /// Item methods
// impl EagleVision {
//     pub fn new_function(&mut self, name: Option<(Entity, impl Into<String>)>) -> Entity {
//         Function::spawn(self, name)
//     }

//     pub fn function<'ev>(&'ev self, function: Entity) -> EagleVisionFunction<'ev> {
//         EagleVisionFunction::new(self, function)
//     }

//     pub fn function_mut<'ev>(&'ev mut self, function: Entity) -> EagleVisionFunctionMut<'ev> {
//         EagleVisionFunctionMut::new(self, function)
//     }

//     pub fn function_parameter<'ev>(&'ev self, parameter: Entity) -> EagleVisionParameter<'ev> {
//         EagleVisionParameter::new(self, parameter)
//     }
// }

// /// Ty methods
// impl EagleVision {
//     pub fn get_core_ty(&self, core_ty: CoreTy) -> Entity {
//         match core_ty {
//             CoreTy::Unknown => TyUnknown::get(self),
//             CoreTy::Unit => TyUnit::get(self),
//             CoreTy::Boolean => TyBoolean::get(self),
//             CoreTy::Int32 => TyInt32::get(self),
//             CoreTy::Float64 => TyFloat64::get(self),
//             CoreTy::Utf8 => TyUtf8::get(self),
//             CoreTy::Function => TyFunction::get(self),
//         }
//     }
// }

// /// World wrapping methods for use within this crate.
// impl EagleVision {
//     #[inline]
//     pub(crate) fn spawn<B: Bundle>(&mut self, bundle: B) -> EntityWorldMut<'_> {
//         self.app.world_mut().spawn(bundle)
//     }

//     #[inline]
//     pub(crate) fn spawn_id<B: Bundle>(&mut self, bundle: B) -> Entity {
//         self.app.world_mut().spawn(bundle).id()
//     }

//     #[inline]
//     pub(crate) fn spawn_empty_id(&mut self) -> Entity {
//         self.app.world_mut().spawn_empty().id()
//     }

//     #[inline]
//     pub(crate) fn insert_into(&mut self, entity: Entity, bundle: impl Bundle) {
//         self.app.world_mut().entity_mut(entity).insert(bundle);
//     }

//     #[inline]
//     pub(crate) fn get_marked_entity<C: SingleEntityMarker>(&self) -> Entity {
//         C::get(&self)
//     }
// }

/// A component that implements `GuaranteedMarkerComponent` must exist on a single
/// entity after initialization of an EagleVision app.
/// 
/// This trait cannot be implemented outside of the crate that defines it.
pub trait GuaranteedMarkerComponent : Component + Sized + EagleVisionSealed {
    fn get(world: &World) -> Entity {
        world.try_query_filtered::<Entity, With<Self>>().unwrap().single(world).unwrap()
    }
}

// pub trait HasComponent<C: Component> {}