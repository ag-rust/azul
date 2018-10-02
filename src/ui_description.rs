use std::{
    cell::RefCell,
    rc::Rc,
    collections::BTreeMap,
};
use {
    FastHashMap,
    css_parser::ParsedCssProperty,
    id_tree::{Arena, NodeId},
    traits::Layout,
    dom::Dom,
    css::{Css, ParsedCss, CssRule, ZIndex, CssDeclaration},
    dom::NodeData,
};

pub struct UiDescription<T: Layout> {
    pub(crate) ui_descr_arena: Rc<RefCell<Arena<NodeData<T>>>>,
    /// ID of the root node of the arena (usually NodeId(0))
    pub(crate) ui_descr_root: NodeId,
    /// This field is created from the Css parser
    pub(crate) styled_nodes: BTreeMap<NodeId, StyledNode>,
    /// In the display list, we take references to the `UiDescription.styled_nodes`
    ///
    /// However, if there is no style, we want to have a default style applied
    /// and the reference to that style has to live as least as long as the `self.styled_nodes`
    /// This is why we need this field here
    pub(crate) default_style_of_node: StyledNode,
    /// The CSS properties that should be overridden for this frame, cloned from the `Css`
    pub(crate) dynamic_css_overrides: FastHashMap<String, ParsedCssProperty>,
}

impl<T: Layout> Clone for UiDescription<T> {
    fn clone(&self) -> Self {
        Self {
            ui_descr_arena: self.ui_descr_arena.clone(),
            ui_descr_root: self.ui_descr_root,
            styled_nodes: self.styled_nodes.clone(),
            default_style_of_node: self.default_style_of_node.clone(),
            dynamic_css_overrides: self.dynamic_css_overrides.clone(),
        }
    }
}

impl<T: Layout> Default for UiDescription<T> {
    fn default() -> Self {
        use dom::NodeType;
        let default_dom = Dom::new(NodeType::Div);
        let default_css = Css::empty();
        Self::from_dom(&default_dom, &default_css)
    }
}

impl<T: Layout> UiDescription<T> {
    /// Applies the CSS styles to the nodes calculated from the `layout_screen`
    /// function and calculates the final display list that is submitted to the
    /// renderer.
    pub fn from_dom(dom: &Dom<T>, style: &Css) -> Self
    {
        ::css::match_dom_css_selectors(dom.root, &dom.arena, &ParsedCss::from_css(style), style, ZIndex(0))
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct StyledNode {
    /// The z-index level that we are currently on, 0 by default
    pub(crate) z_level: ZIndex,
    /// The CSS constraints, after the cascading step
    pub(crate) css_constraints: CssConstraintList
}

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct CssConstraintList {
    pub(crate) list: Vec<CssDeclaration>
}

impl CssConstraintList {
    #[inline]
    pub(crate) fn push_rule(&mut self, rule: &CssRule) {
        self.list.push(rule.declaration.1.clone());
    }
}

// Empty test, for some reason codecov doesn't detect any files (and therefore
// doesn't report codecov % correctly) except if they have at least one test in
// the file. This is an empty test, which should be updated later on
#[test]
fn __codecov_test_ui_description_file() {

}