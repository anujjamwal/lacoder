//! Per-workspace registry of running / past agent sessions.

use std::rc::Rc;

use floem::reactive::{RwSignal, Scope, SignalUpdate, SignalWith};

use crate::{
    agent::session::CoderSession,
    id::{AssistantSessionId, CoderSessionId},
};

#[derive(Clone)]
pub struct AgentRegistry {
    pub coders: RwSignal<im::HashMap<CoderSessionId, Rc<CoderSession>>>,
    pub active_coder: RwSignal<Option<CoderSessionId>>,
    pub active_assistant: RwSignal<Option<AssistantSessionId>>,
}

impl AgentRegistry {
    pub fn new(cx: Scope) -> Self {
        Self {
            coders: cx.create_rw_signal(im::HashMap::new()),
            active_coder: cx.create_rw_signal(None),
            active_assistant: cx.create_rw_signal(None),
        }
    }

    pub fn insert_coder(&self, session: Rc<CoderSession>) {
        let id = session.id;
        self.coders.update(|map| {
            map.insert(id, session);
        });
    }

    pub fn get_coder(&self, id: CoderSessionId) -> Option<Rc<CoderSession>> {
        self.coders.with_untracked(|map| map.get(&id).cloned())
    }
}
