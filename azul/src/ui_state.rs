use std::{
    fmt,
    collections::BTreeMap,
};
use azul_css::CssProperty;
use {
    app::RuntimeError,
    FastHashMap,
    window::{WindowInfo, WindowId},
    traits::Layout,
    dom::{Callback, Dom, TagId, EventFilter, NotEventFilter, WindowEventFilter, DesktopEventFilter, TabIndex},
    app_state::AppState,
    id_tree::NodeId,
    style::HoverGroup,
    default_callbacks::DefaultCallbackId,
};

pub struct UiState<T: Layout> {
    /// The actual DOM, rendered from the .layout() function
    pub dom: Dom<T>,
    /// Tag IDs that were generated by
    pub tag_ids_to_callbacks: BTreeMap<TagId, BTreeMap<EventFilter, Callback<T>>>,
    pub tag_ids_to_default_callbacks: BTreeMap<TagId, BTreeMap<EventFilter, DefaultCallbackId>>,
    pub tab_index_tags: BTreeMap<TagId, (NodeId, TabIndex)>,
    pub draggable_tags: BTreeMap<TagId, NodeId>,
    pub tag_ids_to_node_ids: BTreeMap<TagId, NodeId>,
    /// One node can only have one tag, but one tag can be inserted in more than one map.
    pub node_ids_to_tag_ids: BTreeMap<NodeId, TagId>,
    /// The style properties that should be overridden for this frame, cloned from the `Css`
    pub dynamic_css_overrides: BTreeMap<NodeId, FastHashMap<String, CssProperty>>,
    /// Stores all tags for nodes that need to activate on a `:hover` or `:active` event.
    pub tag_ids_to_hover_active_states: BTreeMap<TagId, (NodeId, HoverGroup)>,
    /// Not-callbacks (callbacks that fire when an item is NOT hovered or focused)
    /// are seperated from the DOM, since they don't create hit-testable tag IDs themselves
    pub not_callbacks: BTreeMap<NodeId, BTreeMap<NotEventFilter, Callback<T>>>,
    pub not_default_callbacks: BTreeMap<NodeId, BTreeMap<NotEventFilter, DefaultCallbackId>>,
    /// Callbacks that are fired on window events (not attached to any item,
    /// but to the whole window), are seperated from the DOM itself
    pub window_callbacks: BTreeMap<NodeId, BTreeMap<WindowEventFilter, Callback<T>>>,
    pub window_default_callbacks: BTreeMap<NodeId, BTreeMap<WindowEventFilter, DefaultCallbackId>>,
    /// Same as `window_callbacks`, but for desktop events
    pub desktop_callbacks: BTreeMap<NodeId, BTreeMap<DesktopEventFilter, Callback<T>>>,
    pub desktop_default_callbacks: BTreeMap<NodeId, BTreeMap<DesktopEventFilter, DefaultCallbackId>>,
}

impl<T: Layout> fmt::Debug for UiState<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
            "UiState {{ \
                \tdom: {:?}, \
                \ttag_ids_to_callbacks: {:?}, \
                \ttag_ids_to_default_callbacks: {:?}, \
                \ttab_index_tags: {:?}, \
                \tdraggable_tags: {:?}, \
                \tnode_ids_to_tag_ids: {:?} \
                \ttag_ids_to_node_ids: {:?} \
                \ttag_ids_to_hover_active_states: {:?} \
                \tnot_callbacks: {:?} \
                \tnot_default_callbacks: {:?} \
                \twindow_callbacks: {:?} \
                \twindow_default_callbacks: {:?} \
                \tdesktop_callbacks: {:?} \
                \tdesktop_default_callbacks: {:?} \
            }}",
            self.dom,
            self.tag_ids_to_callbacks,
            self.tag_ids_to_default_callbacks,
            self.tab_index_tags,
            self.draggable_tags,
            self.node_ids_to_tag_ids,
            self.tag_ids_to_node_ids,
            self.tag_ids_to_hover_active_states,
            self.not_callbacks,
            self.not_default_callbacks,
            self.window_callbacks,
            self.window_default_callbacks,
            self.desktop_callbacks,
            self.desktop_default_callbacks,
        )
    }
}

impl<T: Layout> UiState<T> {
    #[allow(unused_imports, unused_variables)]
    pub(crate) fn from_app_state(app_state: &mut AppState<T>, window_id: &WindowId)
    -> Result<Self, RuntimeError<T>>
    {
        use dom::{Dom, On, NodeType};
        use std::sync::atomic::Ordering;
        use app::RuntimeError::*;

        let mut fake_window = app_state.windows.get_mut(window_id).ok_or(WindowIndexError)?;
        let window_info = WindowInfo {
            window: &mut fake_window,
            resources: &app_state.resources,
        };

        // Only shortly lock the data to get the dom out
        let dom: Dom<T> = {
            let dom_lock = app_state.data.lock().unwrap();
            #[cfg(test)]{
                Dom::<T>::new(NodeType::Div)
            }

            #[cfg(not(test))]{
                dom_lock.layout(window_info)
            }
        };

        Ok(dom.into_ui_state())
    }

    pub fn create_tags_for_hover_nodes(&mut self, hover_nodes: &BTreeMap<NodeId, HoverGroup>) {
        use dom::new_tag_id;
        for (hover_node_id, hover_group) in hover_nodes {
            let hover_tag = match self.node_ids_to_tag_ids.get(hover_node_id) {
                Some(tag_id) => *tag_id,
                None => new_tag_id(),
            };

            self.node_ids_to_tag_ids.insert(*hover_node_id, hover_tag);
            self.tag_ids_to_node_ids.insert(hover_tag, *hover_node_id);
            self.tag_ids_to_hover_active_states.insert(hover_tag, (*hover_node_id, *hover_group));
        }
    }
}
