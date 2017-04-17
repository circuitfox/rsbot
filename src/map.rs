use std::fmt;
use std::slice;

use pathfinding::astar;
use petgraph;
use petgraph::graph;
use petgraph::visit::EdgeRef;
use serde;
use serde::de;
use serde::ser::SerializeStruct;

use {Command, Direction};

#[derive(Debug, Default)]
pub struct Map(petgraph::Graph<bool, Direction, petgraph::Undirected>);

impl Map {
    fn from_nodes_edges(nodes: Vec<bool>, edges: Vec<Edge<Direction>>) -> Self {
        let mut graph = petgraph::Graph::with_capacity(nodes.len(), edges.len());
        for n in nodes {
            graph.add_node(n);
        }
        graph.extend_with_edges(edges);
        Map(graph)
    }

    fn nodes(&self) -> NodeWeights<bool> {
        NodeWeights { nodes: self.0.raw_nodes().iter() }
    }

    fn edges(&self) -> graph::EdgeReferences<Direction> {
        self.0.edge_references()
    }

    // TODO: Error handling here
    pub fn path(&self) -> Path {
        let mut nodes = self.0.node_indices();
        let first = nodes.next().expect("Non-empty graph");
        let path = astar(&first, |n| self.neighbors(n), |_| 0, |n| self.0[*n])
            .expect("No path")
            .0;
        Path(path.windows(2)
            .map(|n| {
                // If this returns None then our pathing should have failed
                let edge = self.0.find_edge(n[0], n[1]).expect("Invalid graph");
                Edge {
                    nodes: (n[0].index() as u32, n[1].index() as u32),
                    weight: self.0[edge],
                }
            })
            .collect::<Vec<_>>())
    }

    fn neighbors(&self, node: &graph::NodeIndex) -> Vec<(graph::NodeIndex, u32)> {
        let mut neighbors = self.0.neighbors(*node).detach();
        let mut nvec = vec![];
        while let Some((edge, target)) = neighbors.next(&self.0) {
            // We arrive at these edge costs by this calculation:
            // Moving forward or backward is a single movement action in the given direction
            // Moving left or right requires two actions: a turn in the given direction,
            // and then a forward movement
            let cost = match self.0[edge] {
                Direction::Forward | Direction::Backward => 1,
                Direction::Left | Direction::Right => 2,
            };
            nvec.push((target, cost));
        }
        nvec
    }
}

#[derive(Debug)]
pub struct Path(Vec<Edge<Direction>>);

impl Path {
    pub fn into_commands(self) -> Vec<Command> {
        let mut command_vec = vec![];
        for edge in self.0 {
            match edge.weight {
                d @ Direction::Forward |
                d @ Direction::Backward => {
                    command_vec.push(Command::Move(d));
                }
                d @ Direction::Left |
                d @ Direction::Right => {
                    command_vec.push(Command::Move(d));
                    command_vec.push(Command::Move(Direction::Forward));
                }
            }
        }
        command_vec.push(Command::Stop);
        command_vec
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Edge<E> {
    nodes: (u32, u32),
    weight: E,
}

impl<E> petgraph::IntoWeightedEdge<E> for Edge<E> {
    type NodeId = u32;
    fn into_weighted_edge(self) -> (Self::NodeId, Self::NodeId, E) {
        (self.nodes.0, self.nodes.1, self.weight)
    }
}

impl<E> From<graph::Edge<E>> for Edge<E> {
    fn from(edge: graph::Edge<E>) -> Self {
        Edge {
            nodes: (edge.source().index() as u32, edge.target().index() as u32),
            weight: edge.weight,
        }
    }
}

struct NodeWeights<'a, N: 'a> {
    nodes: slice::Iter<'a, graph::Node<N>>,
}

impl<'a, N> Iterator for NodeWeights<'a, N> {
    type Item = &'a N;

    fn next(&mut self) -> Option<Self::Item> {
        self.nodes.next().map(|n| &n.weight)
    }
}

impl serde::Serialize for Map {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        let nodes = self.nodes().collect::<Vec<_>>();
        let edges = self.edges()
            .map(|e| {
                Edge {
                    nodes: (e.source().index() as u32, e.target().index() as u32),
                    weight: e.weight(),
                }
            })
            .collect::<Vec<_>>();
        let mut struc = serializer.serialize_struct("Map", 2)?;
        struc.serialize_field("nodes", &nodes)?;
        struc.serialize_field("edges", &edges)?;
        struc.end()
    }
}

impl serde::Deserialize for Map {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        const FIELDS: &'static [&'static str] = &["nodes", "edges"];

        enum Field {
            Nodes,
            Edges,
        }

        struct FieldVisitor;

        impl de::Visitor for FieldVisitor {
            type Value = Field;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("`nodes` or `edges`")
            }

            fn visit_str<E>(self, value: &str) -> Result<Field, E>
                where E: de::Error
            {
                match value {
                    "nodes" => Ok(Field::Nodes),
                    "edges" => Ok(Field::Edges),
                    _ => Err(de::Error::unknown_field(value, FIELDS)),
                }
            }
        }

        impl serde::Deserialize for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: serde::Deserializer
            {
                deserializer.deserialize_struct_field(FieldVisitor)
            }
        }

        struct MapVisitor;

        impl de::Visitor for MapVisitor {
            type Value = Map;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Map")
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Map, V::Error>
                where V: de::MapVisitor
            {
                let mut nodes = None;
                let mut edges = None;
                while let Some(key) = visitor.visit_key()? {
                    match key {
                        Field::Nodes => {
                            if nodes.is_some() {
                                return Err(de::Error::duplicate_field("nodes"));
                            }
                            nodes = Some(visitor.visit_value()?);
                        }
                        Field::Edges => {
                            if edges.is_some() {
                                return Err(de::Error::duplicate_field("edges"));
                            }
                            edges = Some(visitor.visit_value()?);
                        }
                    }
                }
                let nodes = match nodes {
                    Some(nodes) => nodes,
                    None => return Err(de::Error::missing_field("nodes")),
                };
                let edges = match edges {
                    Some(edges) => edges,
                    None => return Err(de::Error::missing_field("edges")),
                };
                Ok(Map::from_nodes_edges(nodes, edges))
            }
        }

        deserializer.deserialize_struct("Map", FIELDS, MapVisitor)
    }
}
