use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use ambassador::{delegatable_trait, Delegate};
pub use hex_color::HexColor;
use serde::{Deserialize, Deserializer, Serialize};
use url::Url;

pub type NodeId = String;
pub type EdgeId = String;
pub type PxCoord = i64;
pub type PxLength = u64;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Canvas {
    #[serde(deserialize_with = "deserialize_null_default")]
    nodes: Vec<Node>,
    #[serde(deserialize_with = "deserialize_null_default")]
    edges: Vec<Edge>,
}

/// from https://github.com/serde-rs/serde/issues/1098
fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

impl Canvas {
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn nodes_mut(&mut self) -> &mut Vec<Node> {
        self.nodes.as_mut()
    }

    pub fn edges_mut(&mut self) -> &mut Vec<Edge> {
        self.edges.as_mut()
    }

    /// Find any nodes which are referred to by edges but not in the nodes container.
    pub fn unknown_nodes(&self) -> HashSet<&str> {
        let mut out = self.edges.iter().fold(HashSet::default(), |mut ids, edge| {
            ids.insert(edge.from_node().as_str());
            ids.insert(edge.to_node().as_str());
            ids
        });
        for node in self.nodes.iter() {
            out.remove(node.id().as_str());
        }
        out
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Location {
    pub x: PxCoord,
    pub y: PxCoord,
}

impl Location {
    pub fn new(x: PxCoord, y: PxCoord) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Dimensions {
    pub width: PxLength,
    pub height: PxLength,
}

impl Dimensions {
    pub fn new(width: PxLength, height: PxLength) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericNode {
    id: NodeId,
    #[serde(flatten)]
    location: Location,
    #[serde(flatten)]
    dimensions: Dimensions,
    color: Option<Color>,
}

impl GenericNode {
    pub fn new(
        id: NodeId,
        location: Location,
        dimensions: Dimensions,
        color: Option<Color>,
    ) -> Self {
        Self {
            id,
            location,
            dimensions,
            color,
        }
    }
}

#[delegatable_trait]
pub trait GenericNodeInfo {
    fn id(&self) -> &NodeId;
    fn location(&self) -> &Location;
    fn dimensions(&self) -> &Dimensions;
    fn color(&self) -> &Option<Color>;
}

impl GenericNodeInfo for GenericNode {
    fn id(&self) -> &NodeId {
        &self.id
    }

    fn location(&self) -> &Location {
        &self.location
    }

    fn dimensions(&self) -> &Dimensions {
        &self.dimensions
    }

    fn color(&self) -> &Option<Color> {
        &self.color
    }
}

#[derive(Debug, Clone, Delegate, Serialize, Deserialize)]
#[delegate(GenericNodeInfo, target = "generic")]
pub struct TextNode {
    #[serde(flatten)]
    generic: GenericNode,
    text: String,
}

impl TextNode {
    pub fn new(
        id: NodeId,
        location: Location,
        dimensions: Dimensions,
        color: Option<Color>,
        text: String,
    ) -> Self {
        Self {
            generic: GenericNode::new(id, location, dimensions, color),
            text,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Debug, Clone, Delegate, Serialize, Deserialize)]
#[delegate(GenericNodeInfo, target = "generic")]
pub struct FileNode {
    #[serde(flatten)]
    generic: GenericNode,
    file: PathBuf,
    subpath: Option<String>,
}

impl FileNode {
    pub fn new(
        id: NodeId,
        location: Location,
        dimensions: Dimensions,
        color: Option<Color>,
        file: PathBuf,
        subpath: Option<String>,
    ) -> Self {
        Self {
            generic: GenericNode::new(id, location, dimensions, color),
            file,
            subpath,
        }
    }

    pub fn file(&self) -> &Path {
        &self.file
    }

    pub fn subpath(&self) -> Option<&str> {
        self.subpath.as_deref()
    }
}

#[derive(Debug, Clone, Delegate, Serialize, Deserialize)]
#[delegate(GenericNodeInfo, target = "generic")]
pub struct LinkNode {
    #[serde(flatten)]
    generic: GenericNode,
    url: Url,
}

impl LinkNode {
    pub fn new(
        id: NodeId,
        location: Location,
        dimensions: Dimensions,
        color: Option<Color>,
        url: Url,
    ) -> Self {
        Self {
            generic: GenericNode::new(id, location, dimensions, color),
            url,
        }
    }
    pub fn url(&self) -> &Url {
        &self.url
    }
}

#[derive(Debug, Clone, Delegate, Serialize, Deserialize)]
#[delegate(GenericNodeInfo, target = "generic")]
#[serde(rename_all = "camelCase")]
pub struct GroupNode {
    #[serde(flatten)]
    generic: GenericNode,
    label: Option<String>,
    background: Option<PathBuf>,
    background_style: Option<BackgroundStyle>,
}

impl GroupNode {
    pub fn new(
        id: NodeId,
        location: Location,
        dimensions: Dimensions,
        color: Option<Color>,
        label: Option<String>,
        background: Option<PathBuf>,
        background_style: Option<BackgroundStyle>,
    ) -> Self {
        Self {
            generic: GenericNode::new(id, location, dimensions, color),
            label,
            background,
            background_style,
        }
    }
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    pub fn background(&self) -> Option<&Path> {
        self.background.as_deref()
    }

    pub fn background_style(&self) -> Option<BackgroundStyle> {
        self.background_style
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BackgroundStyle {
    Cover,
    Ratio,
    Repeat,
}

#[derive(Debug, Clone, Delegate, Serialize, Deserialize)]
#[delegate(GenericNodeInfo)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Node {
    Text(TextNode),
    File(FileNode),
    Link(LinkNode),
    Group(GroupNode),
}

impl From<TextNode> for Node {
    fn from(value: TextNode) -> Self {
        Self::Text(value)
    }
}

impl From<FileNode> for Node {
    fn from(value: FileNode) -> Self {
        Self::File(value)
    }
}

impl From<LinkNode> for Node {
    fn from(value: LinkNode) -> Self {
        Self::Link(value)
    }
}

impl From<GroupNode> for Node {
    fn from(value: GroupNode) -> Self {
        Self::Group(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Edge {
    id: EdgeId,
    from_node: NodeId,
    from_side: Option<Side>,
    from_end: Option<EndStyle>,
    to_node: NodeId,
    to_side: Option<Side>,
    to_end: Option<EndStyle>,
    color: Option<Color>,
    label: Option<String>,
}

pub struct Terminal {
    node_id: NodeId,
    side: Option<Side>,
    end_style: Option<EndStyle>,
}

impl Terminal {
    pub fn new(node_id: NodeId, side: Option<Side>, end_style: Option<EndStyle>) -> Self {
        Self {
            node_id,
            side,
            end_style,
        }
    }
}

impl Edge {
    pub fn new(
        id: EdgeId,
        from: Terminal,
        to: Terminal,
        color: Option<Color>,
        label: Option<String>,
    ) -> Self {
        Self {
            id,
            from_node: from.node_id,
            from_side: from.side,
            from_end: from.end_style,
            to_node: to.node_id,
            to_side: to.side,
            to_end: to.end_style,
            color,
            label,
        }
    }

    pub fn id(&self) -> &EdgeId {
        &self.id
    }

    pub fn from_node(&self) -> &NodeId {
        &self.from_node
    }

    pub fn from_side(&self) -> Option<Side> {
        self.from_side
    }

    pub fn from_end(&self) -> Option<EndStyle> {
        self.from_end
    }

    pub fn to_node(&self) -> &NodeId {
        &self.to_node
    }

    pub fn to_side(&self) -> Option<Side> {
        self.to_side
    }

    pub fn to_end(&self) -> Option<EndStyle> {
        self.to_end
    }

    pub fn color(&self) -> Option<Color> {
        self.color
    }

    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EndStyle {
    None,
    Arrow,
}

impl Default for EndStyle {
    fn default() -> Self {
        Self::None
    }
}

impl EndStyle {
    /// The [EndStyle::None] end type becomes [None],
    /// all other styles become `Some(EndStyle)`.
    pub fn into_option(self) -> Option<Self> {
        match self {
            Self::None => None,
            s => Some(s),
        }
    }
}

impl From<Option<EndStyle>> for EndStyle {
    fn from(value: Option<EndStyle>) -> Self {
        value.unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Color {
    Preset(PresetColor),
    Hex(HexColor),
}

impl Default for Color {
    fn default() -> Self {
        Self::Hex(HexColor::WHITE)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresetColor {
    Red = 1,
    Orange = 2,
    Yellow = 3,
    Green = 4,
    Cyan = 5,
    Purple = 6,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils;

    fn read_sample(stem: &str) -> String {
        test_utils::read_sample(test_utils::Version::V1_0, stem)
    }

    #[test]
    fn can_deser() {
        let sample = read_sample("sample");
        let _canvas: Canvas = serde_json::from_str(&sample).unwrap();
    }

    #[test]
    fn can_ser() {
        let mut canvas = Canvas::default();
        let n = canvas.nodes_mut();

        n.push(
            TextNode::new(
                "mytextnode".to_string(),
                Location::new(1, 2),
                Dimensions::new(10, 20),
                Some(Color::Preset(PresetColor::Purple)),
                "Greetings to my lovely new node".to_string(),
            )
            .into(),
        );

        n.push(
            LinkNode::new(
                "mylinknode".to_string(),
                Location::new(100, 200),
                Dimensions::new(10, 5),
                Some(Color::Hex(HexColor::rgb(255, 0, 0))),
                "https://jsoncanvas.org/".try_into().unwrap(),
            )
            .into(),
        );

        let e = canvas.edges_mut();

        e.push(Edge::new(
            "myedge".to_string(),
            Terminal::new("mytextnode".to_string(), Some(Side::Right), None),
            Terminal::new(
                "mylinknode".to_string(),
                Some(Side::Left),
                Some(EndStyle::Arrow),
            ),
            Some(Color::Preset(PresetColor::Cyan)),
            Some("Look out, it's an edge!".to_string()),
        ));

        let s = serde_json::to_string_pretty(&canvas).unwrap();
        let _canvas2: Canvas = serde_json::from_str(&s).unwrap();
    }
}
