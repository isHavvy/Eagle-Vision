//! The Eagle Vision Source controller
//! 
//! God object that actually holds the source and maintains its invariants.

// #[TODO(Havvy)]: Don't return bevy Entities. Instead return wrappers that guarantee more information.

use std::cell::RefCell;
use std::num::NonZeroU16;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::TryRecvError;

use crate::bevy::prelude::*;
use crate::engine_plugin::{EagleVisionEnginePlugin, WorkPending};
use crate::schedule::{PostStartup, Startup, Update};
use crate::subscription::SubscriptionPlugin;
use bevy::log::{Level, LogPlugin};

use bevy::platform::collections::HashMap;
use eagle_squak::aspect::AspectKind;
use eagle_squak::id::{AspectedId, EagleVisionId, SourceId};
use eagle_squak::id_allocator::AspectIdAllocator;
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
use crate::init::{NewProjectSettings, EagleVisionInitializationPlugin};
use crate::op::OpPlugin;
use crate::private::EagleVisionSealed;
use crate::server::{EagleVisionServerPlugin, session::handle_hello};

#[derive(Debug)]
pub struct EagleVision {
    app: App,
}

impl EagleVision {
    pub fn new_project<TransportPlugin: Plugin>(
        settings: NewProjectSettings,
        transport_plugin: TransportPlugin,
    ) -> Self {
        let mut app = App::empty();

        app.add_plugins((
            EagleVisionEnginePlugin,
            EagleVisionInitializationPlugin { settings },
            EagleVisionServerPlugin,
            transport_plugin,
        ));

        app.finish();
        app.cleanup();

        let world = app.world_mut();
        world.run_schedule(Startup);
        world.flush();
        world.clear_trackers();

        world.run_schedule(PostStartup);
        world.flush();
        world.clear_trackers();

        Self { app }
    }

    pub fn new_project_with_single_client(
        settings: NewProjectSettings,
        hello: ClientHello,
    ) -> Result<(), HandshakeError> {
        todo!()
    }

    /// Execute one turn of the language engine.
    /// 
    /// This will read all client requests and send out subscription updates
    /// and other server messages.
    /// 
    /// Long running tasks will make progress and may or may notcomplete.
    pub fn turn(&mut self) {
        let world = self.app.world_mut();
        world.resource::<WorkPending>().clear();
        world.run_schedule(Update);
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

// pub struct SingleClientProject {
//     pub engine: EagleVision,
//     pub handle: SingleClientHandle,
//     pub welcome: ServerWelcome,
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
//     #[inline]
//     pub(crate) fn get_marked_entity<C: SingleEntityMarker>(&self) -> Entity {
//         C::get(&self)
//     }
// }

// pub trait HasComponent<C: Component> {}