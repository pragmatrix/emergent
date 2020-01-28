//! Input transactions.

pub fn begin<T>(transaction: T) -> InitialResponse<T>
where
    T: Transaction,
{
    InitialResponse {
        action: InitialAction::Begin(transaction),
        event: None,
    }
}

pub fn neglect<T>() -> InitialResponse<T>
where
    T: Transaction,
{
    InitialResponse {
        action: InitialAction::Neglect,
        event: None,
    }
}

pub fn sustain<OE>() -> UpdateResponse<OE> {
    UpdateResponse::sustain()
}

pub fn commit<OE>() -> UpdateResponse<OE> {
    UpdateResponse::commit()
}

pub fn rollback<OE>() -> UpdateResponse<OE> {
    UpdateResponse::rollback()
}

pub enum InitialAction<T> {
    Begin(T),
    Neglect,
}

pub struct InitialResponse<T>
where
    T: Transaction,
{
    pub action: InitialAction<T>,
    pub event: Option<T::OutputEvent>,
}

impl<T> InitialResponse<T>
where
    T: Transaction,
{
    pub fn with_event(mut self, e: T::OutputEvent) -> Self {
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

pub struct UpdateResponse<OutputEvent> {
    pub action: UpdateAction,
    pub event: Option<OutputEvent>,
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

    fn sustain() -> Self {
        Self::from_action(UpdateAction::Sustain)
    }

    fn commit() -> Self {
        Self::from_action(UpdateAction::Commit)
    }

    fn rollback() -> Self {
        Self::from_action(UpdateAction::Rollback)
    }
}

/// Implement the trait when an input processors needs to:
///
/// - activate in response to an input event.
/// - access and modify external state.
/// - be seen as a transaction regarding to the external state,
///   i.e. supporting a rollback function that undos it.
/// - may optionally return an event at any time.
pub trait Transaction {
    type InputEvent;
    type ViewState;
    type OutputEvent;

    fn update(
        &mut self,
        event: Self::InputEvent,
        state: &mut Self::ViewState,
    ) -> UpdateResponse<Self::OutputEvent>;
}
