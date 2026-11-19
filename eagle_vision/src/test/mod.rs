//! Testing framework for testing EagleVision on top of its
//! [`LanguageServerPlugin`](crate::server::LanguageServerPlugin).

use bevy::prelude::*;
use eagle_squak::id::{EagleVisionId, LocaleId};
use eagle_squak::protocol::*;
use eagle_squak::subscription::{SubscriptionId, SubscriptionTarget};

use crate::aspect::metadata::BaseLocale;
use crate::eaglevision::{EagleVision, GuaranteedMarkerComponent};
use crate::init::EagleVisionInitialSettings;
use crate::server::{IncomingRequests, OutgoingMessages, Sessions};

mod unfiled_tests;

mod readonly;

pub struct TestServer {
    pub ev: EagleVision,
    pub welcome: ServerWelcome,
    next_request: u64,
    next_sub: u32,
}

impl TestServer {
    pub fn new(settings: EagleVisionInitialSettings) -> Self {
        Self::with_hello(settings, ClientHello {
            username: Some("tester".into()),
            locales: vec!["en-US".into()],
            flags: Default::default(),
            protocol_version: PROTOCOL_VERSION,
        }).expect("handshake")
    }

    pub fn with_hello(
        settings: EagleVisionInitialSettings,
        hello: ClientHello,
    ) -> Result<Self, HandshakeError> {
        let mut ev = EagleVision::new_project(settings, false);
        let welcome = ev.handle_hello(hello)?;
        Ok(Self { ev, welcome, next_request: 1, next_sub: 1 })
    }

    fn session_entity(&self) -> Entity {
        self.ev.world().resource::<Sessions>()[self.welcome.session_id]
    }

    pub fn push(&mut self, body: ClientRequestBody) -> RequestId {
        let id = RequestId(self.next_request);
        self.next_request += 1;
        self.ev.world_mut().resource_mut::<IncomingRequests>().0
            .push_back((self.welcome.session_id, ClientRequest { id, body }));
        id
    }

    pub fn turn(&mut self) { self.ev.turn(); }

    pub fn request(&mut self, body: ClientRequestBody) -> RequestId {
        let id = self.push(body);
        self.turn();
        id
    }

    pub fn subscribe(&mut self, spec: SubscriptionTarget) -> (SubscriptionId, RequestId) {
        let sub = SubscriptionId(self.next_sub);
        self.next_sub += 1;
        let req = self.request(ClientRequestBody::Subscribe { id: sub, spec });
        (sub, req)
    }

    pub fn drain(&mut self) -> Vec<ServerMessage> {
        let entity = self.session_entity();
        self.ev.world_mut().get_mut::<OutgoingMessages>(entity)
            .map(|mut o| o.0.drain(..).collect()).unwrap_or_default()
    }

    pub fn base_locale(&self) -> LocaleId {
        let entity = BaseLocale::get(&self.ev.world());
        let raw = *self.ev.world().get::<EagleVisionId>(entity).unwrap();
        LocaleId::from_untyped(raw).unwrap()
    }

    /// Run turns until `pred` matches; panic with everything seen otherwise.
    pub fn wait_for<F>(&mut self, turns: usize, mut pred: F) -> ServerMessage
    where F: FnMut(&ServerMessage) -> bool {
        let mut seen = Vec::new();
        for _ in 0..turns {
            for m in self.drain() {
                if pred(&m) { return m; }
                seen.push(m);
            }
            self.turn();
        }
        panic!("not found within {turns} turns; saw {seen:#?}");
    }
}

// #[test]
// fn project_name() {
//     let builder = EagleVisionBuilder {
//         base_locale: Some("en-US".to_string()),
//         project_name: Some("Test Project".to_string()),
//         ..Default::default()
//     };

//     let mut ev = EagleVision::new(builder);

//     let en = ev.get_locale("en-US").unwrap();

//     let project_name = ev.get_project_name(en);
//     assert_eq!(Some("Test Project"), project_name);

//     let es = ev.insert_locale("es-MX".to_string());

//     let project_name_text = ev.get_project_name_text();
//     ev.upsert_text(project_name_text, es, "Projecta Prueba".to_string());

//     let project_name = ev.get_project_name(en);
//     assert_eq!(Some("Test Project"), project_name);
//     let project_name = ev.get_project_name(es);
//     assert_eq!(Some("Projecta Prueba"), project_name);
// }

// #[test]
// fn print_two_plus_three() {
//     let mut ev = EagleVision::new(Default::default());

//     let exe = ev.new_executable();

//     let two = CoreLiteralValue::Int32(2);
//     let three = CoreLiteralValue::Int32(3);

//     let two_expr = ev.new_literal_expression(two);
//     let three_expr = ev.new_literal_expression(three);
//     let plus_expr = ev.new_operator_expression(MathOperator::Plus, two_expr, three_expr, None);
//     let print_expr = ev.new_print_expression(plus_expr);

//     // let exe_ctx = ev.executable(exe);
//     // let block_expr = exe_ctx.main_block();
//     // ev.push_block_expression_statement(block_expr, print_expr);

//     // let exe_ctx = ev.executable_mut(exe);
//     // let exe = exe_ctx.compile().unwrap();
//     // let exe = exe_ctx.get_compiled(exe).unwrap();

//     // let output = exe.execute().ok().unwrap();
//     // assert_eq!(output, "5\n");
// }

// #[test]
// fn create_and_call_function() {
//     let mut ev = EagleVision::new(Default::default());

//     let en = ev.get_locale("en-US").unwrap();

//     let square = ev.new_function(Some((en, "Square")));
//     let ty_i32 = ev.get_core_ty(CoreTy::Int32);

//     // let p1 = ev.new_binding_expression(BindingExpression::Parameter(0, square));
//     // let p2 = ev.new_binding_expression(BindingExpression::Parameter(0, square));
//     // let mult = ev.new_operator_expression(MathOperator::Mult, p1, p2, None);

//     // let square_ctx = ev.function_mut(square);
//     // square_ctx.set_return_ty(ty_i32);
//     // square_ctx.push_parameter(ty_i32);
//     // square_ctx.replace_body(mult);
//     // let square_path = square_ctx.path_expression();

//     // let exe = ev.new_executable();

//     // let three_expr = ev.new_literal_expression(CoreLiteralValue::Int32(3));
//     // let square_expr = ev.new_path_expression(square_path);
//     // let call_expr = ev.new_call_expression(CallExpression { callable: square_expr, arguments: vec![three_expr] });
//     // let print_expr = ev.new_print_expression(call_expr);
//     // let block_expr = ev.get_executable_main_block(exe);
//     // ev.push_block_expression_statement(block_expr, print_expr);

//     // let exe = ev.compile_executable(exe).unwrap();
//     // let exe = ev.get_executable(exe);

//     // let output = exe.execute().ok().unwrap();
//     // assert_eq!(output, "9\n");
// }