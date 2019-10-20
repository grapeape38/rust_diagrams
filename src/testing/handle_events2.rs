use std::collections::{HashMap};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};
use sdl2::mouse::{Cursor, SystemCursor};

struct AppState {
    display_tree: DisplayTree
}

type ParentCallbackT<T> = Box<Fn(&mut T)>;
type RootCallback = Box<Fn(&mut AppState)>;

enum EventResponse<T> {
    NotHandled,
    Handled,
    ParentCallback<ParentCallbackT<T>>,
    RootCallback<RootCallbackT>
}

enum TrySelfHandle<P> {
    SelfHandled(HandleEvent<P>),
    TryChildren,
}

trait HandleEvent<P> {
    fn handle_event(&mut self, ev: &Event) -> EventResponse<P>;
}

trait DisplayNode<P> {
    fn try_handle_event(&mut self, ev: &Event) -> TrySelfHandle;
    fn children_mut(&mut self) -> impl Iterator<Box<DisplayNode<Self>>>;
}

impl HandleEvent<P> for DisplayNode<P> {
    fn handle_event(&mut self, ev: &Event) -> EventResponse<P> {
        if let SelfHandled(handled) = try_handle_event(self, ev) {
            return handled;
        }
        for c in children_mut {
            match c.handle_event(ev) => {
                EventResponse::Handled => return Handled,
                EventResponse::ParentCallback(callback) => {
                    callback(self);
                    return Handled;
                },
                _ => {} 
            }
        }
        return NotHandled;
    }
}

struct DisplayTree {

}