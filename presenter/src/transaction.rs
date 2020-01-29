//! Input transactions.

pub fn begin<In, S, U, Out>(u: U) -> InitialResponse<U, Out>
where
    U: FnMut(In, &mut S) -> UpdateResponse<Out>,
{
    InitialResponse {
        action: InitialAction::Begin(u),
        event: None,
    }
}

pub fn neglect<U, Out>() -> InitialResponse<U, Out> {
    InitialResponse {
        action: InitialAction::Neglect,
        event: None,
    }
}

pub fn sustain<Out>() -> UpdateResponse<Out> {
    UpdateResponse::from_action(UpdateAction::Sustain)
}

pub fn commit<Out>() -> UpdateResponse<Out> {
    UpdateResponse::from_action(UpdateAction::Commit)
}

pub fn rollback<OE>() -> UpdateResponse<OE> {
    UpdateResponse::from_action(UpdateAction::Rollback)
}

pub enum InitialAction<T> {
    Begin(T),
    Neglect,
}

pub struct InitialResponse<U, Out> {
    pub action: InitialAction<U>,
    pub event: Option<Out>,
}

impl<U, Out> InitialResponse<U, Out> {
    pub fn with_event(mut self, e: Out) -> Self {
        if let Some(ref _e) = self.event {
            debug_assert!(false, "would overwrite transaction event");
        }
        self.event = Some(e);
        self
    }
}

pub enum UpdateAction {
    // would use continue here, but continue is a reserved word.
    Sustain,
    Commit,
    Rollback,
}

pub struct UpdateResponse<Out> {
    pub action: UpdateAction,
    pub event: Option<Out>,
}

impl<OutputEvent> UpdateResponse<OutputEvent> {
    pub fn with_event(mut self, ev: OutputEvent) -> Self {
        if let Some(ref _e) = self.event {
            debug_assert!(false, "would overwrite transaction event");
        }
        self.event = Some(ev);
        self
    }

    // todo: may implement From, but what about the generic return type?
    fn from_action(action: UpdateAction) -> Self {
        Self {
            action,
            event: None,
        }
    }
}
