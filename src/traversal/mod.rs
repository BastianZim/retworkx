// Licensed under the Apache License, Version 2.0 (the "License"); you may
// not use this file except in compliance with the License. You may obtain
// a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations
// under the License.

mod bfs_visit;

use bfs_visit::{bfs_handler, PyBfsVisitor};
use retworkx_core::traversal::{breadth_first_search, dfs_edges};

use super::{digraph, graph, iterators};

use hashbrown::HashSet;

use pyo3::prelude::*;
use pyo3::Python;

use petgraph::algo;
use petgraph::graph::NodeIndex;
use petgraph::visit::{Bfs, NodeCount, Reversed};

use crate::iterators::EdgeList;

/// Get edge list in depth first order
///
/// :param PyDiGraph graph: The graph to get the DFS edge list from
/// :param int source: An optional node index to use as the starting node
///     for the depth-first search. The edge list will only return edges in
///     the components reachable from this index. If this is not specified
///     then a source will be chosen arbitrarly and repeated until all
///     components of the graph are searched.
///
/// :returns: A list of edges as a tuple of the form ``(source, target)`` in
///     depth-first order
/// :rtype: EdgeList
#[pyfunction]
#[pyo3(text_signature = "(graph, /, source=None)")]
fn digraph_dfs_edges(
    graph: &digraph::PyDiGraph,
    source: Option<usize>,
) -> EdgeList {
    EdgeList {
        edges: dfs_edges(&graph.graph, source.map(NodeIndex::new)),
    }
}

/// Get edge list in depth first order
///
/// :param PyGraph graph: The graph to get the DFS edge list from
/// :param int source: An optional node index to use as the starting node
///     for the depth-first search. The edge list will only return edges in
///     the components reachable from this index. If this is not specified
///     then a source will be chosen arbitrarly and repeated until all
///     components of the graph are searched.
///
/// :returns: A list of edges as a tuple of the form ``(source, target)`` in
///     depth-first order
/// :rtype: EdgeList
#[pyfunction]
#[pyo3(text_signature = "(graph, /, source=None)")]
fn graph_dfs_edges(graph: &graph::PyGraph, source: Option<usize>) -> EdgeList {
    EdgeList {
        edges: dfs_edges(&graph.graph, source.map(NodeIndex::new)),
    }
}

/// Return successors in a breadth-first-search from a source node.
///
/// The return format is ``[(Parent Node, [Children Nodes])]`` in a bfs order
/// from the source node provided.
///
/// :param PyDiGraph graph: The DAG to get the bfs_successors from
/// :param int node: The index of the dag node to get the bfs successors for
///
/// :returns: A list of nodes's data and their children in bfs order. The
///     BFSSuccessors class that is returned is a custom container class that
///     implements the sequence protocol. This can be used as a python list
///     with index based access.
/// :rtype: BFSSuccessors
#[pyfunction]
#[pyo3(text_signature = "(graph, node, /)")]
fn bfs_successors(
    py: Python,
    graph: &digraph::PyDiGraph,
    node: usize,
) -> iterators::BFSSuccessors {
    let index = NodeIndex::new(node);
    let mut bfs = Bfs::new(&graph.graph, index);
    let mut out_list: Vec<(PyObject, Vec<PyObject>)> =
        Vec::with_capacity(graph.node_count());
    while let Some(nx) = bfs.next(&graph.graph) {
        let children = graph
            .graph
            .neighbors_directed(nx, petgraph::Direction::Outgoing);
        let mut succesors: Vec<PyObject> = Vec::new();
        for succ in children {
            succesors
                .push(graph.graph.node_weight(succ).unwrap().clone_ref(py));
        }
        if !succesors.is_empty() {
            out_list.push((
                graph.graph.node_weight(nx).unwrap().clone_ref(py),
                succesors,
            ));
        }
    }
    iterators::BFSSuccessors {
        bfs_successors: out_list,
    }
}

/// Return the ancestors of a node in a graph.
///
/// This differs from :meth:`PyDiGraph.predecessors` method  in that
/// ``predecessors`` returns only nodes with a direct edge into the provided
/// node. While this function returns all nodes that have a path into the
/// provided node.
///
/// :param PyDiGraph graph: The graph to get the descendants from
/// :param int node: The index of the graph node to get the ancestors for
///
/// :returns: A list of node indexes of ancestors of provided node.
/// :rtype: list
#[pyfunction]
#[pyo3(text_signature = "(graph, node, /)")]
fn ancestors(graph: &digraph::PyDiGraph, node: usize) -> HashSet<usize> {
    let index = NodeIndex::new(node);
    let mut out_set: HashSet<usize> = HashSet::new();
    let reverse_graph = Reversed(&graph.graph);
    let res = algo::dijkstra(reverse_graph, index, None, |_| 1);
    for n in res.keys() {
        let n_int = n.index();
        out_set.insert(n_int);
    }
    out_set.remove(&node);
    out_set
}

/// Return the descendants of a node in a graph.
///
/// This differs from :meth:`PyDiGraph.successors` method in that
/// ``successors``` returns only nodes with a direct edge out of the provided
/// node. While this function returns all nodes that have a path from the
/// provided node.
///
/// :param PyDiGraph graph: The graph to get the descendants from
/// :param int node: The index of the graph node to get the descendants for
///
/// :returns: A list of node indexes of descendants of provided node.
/// :rtype: list
#[pyfunction]
#[pyo3(text_signature = "(graph, node, /)")]
fn descendants(graph: &digraph::PyDiGraph, node: usize) -> HashSet<usize> {
    let index = NodeIndex::new(node);
    let mut out_set: HashSet<usize> = HashSet::new();
    let res = algo::dijkstra(&graph.graph, index, None, |_| 1);
    for n in res.keys() {
        let n_int = n.index();
        out_set.insert(n_int);
    }
    out_set.remove(&node);
    out_set
}

/// Breadth-first traversal of a directed graph.
///
/// The pseudo-code for the BFS algorithm is listed below, with the annotated
/// event points, for which the given visitor object will be called with the
/// appropriate method.
///
/// ::
///
///     BFS(G, s)
///       for each vertex u in V
///           color[u] := WHITE
///       end for
///       color[s] := GRAY
///       EQUEUE(Q, s)                             discover vertex s
///       while (Q != Ø)
///           u := DEQUEUE(Q)
///           for each vertex v in Adj[u]          (u,v) is a tree edge
///               if (color[v] = WHITE)
///                   color[v] = GRAY
///               else                             (u,v) is a non - tree edge
///                   if (color[v] = GRAY)         (u,v) has a gray target
///                       ...
///                   else if (color[v] = BLACK)   (u,v) has a black target
///                       ...
///           end for
///           color[u] := BLACK                    finish vertex u
///       end while
///
/// If an exception is raised inside the callback function, the graph traversal
/// will be stopped immediately. You can exploit this to exit early by raising a
/// :class:`~retworkx.visit.StopSearch` exception, in which case the search function
/// will return but without raising back the exception. You can also prune part of the
/// search tree by raising :class:`~retworkx.visit.PruneSearch`.
///
/// In the following example we keep track of the tree edges:
///
/// .. jupyter-execute::
///
///        import retworkx
///        from retworkx.visit import BFSVisitor
///   
///        class TreeEdgesRecorder(BFSVisitor):
///
///            def __init__(self):
///                self.edges = []
///
///            def tree_edge(self, edge):
///                self.edges.append(edge)
///
///        graph = retworkx.PyDiGraph()
///        graph.extend_from_edge_list([(1, 3), (0, 1), (2, 1), (0, 2)])
///        vis = TreeEdgesRecorder()
///        retworkx.bfs_search(graph, [0], vis)
///        print('Tree edges:', vis.edges)
///
/// .. note::
///
///     Graph can **not** be mutated while traversing.
///
/// :param PyDiGraph graph: The graph to be used.
/// :param List[int] source: An optional list of node indices to use as the starting nodes
///     for the breadth-first search. If this is not specified then a source
///     will be chosen arbitrarly and repeated until all components of the
///     graph are searched.
/// :param visitor: A visitor object that is invoked at the event points inside the
///     algorithm. This should be a subclass of :class:`~retworkx.visit.BFSVisitor`.
#[pyfunction]
#[pyo3(text_signature = "(graph, source, visitor)")]
pub fn digraph_bfs_search(
    py: Python,
    graph: &digraph::PyDiGraph,
    source: Option<Vec<usize>>,
    visitor: PyBfsVisitor,
) -> PyResult<()> {
    let starts: Vec<_> = match source {
        Some(nx) => nx.into_iter().map(NodeIndex::new).collect(),
        None => graph.graph.node_indices().collect(),
    };

    breadth_first_search(&graph.graph, starts, |event| {
        bfs_handler(py, &visitor, event)
    })?;

    Ok(())
}

/// Breadth-first traversal of an undirected graph.
///
/// The pseudo-code for the BFS algorithm is listed below, with the annotated
/// event points, for which the given visitor object will be called with the
/// appropriate method.
///
/// ::
///
///     BFS(G, s)
///       for each vertex u in V
///           color[u] := WHITE
///       end for
///       color[s] := GRAY
///       EQUEUE(Q, s)                             discover vertex s
///       while (Q != Ø)
///           u := DEQUEUE(Q)
///           for each vertex v in Adj[u]          (u,v) is a tree edge
///               if (color[v] = WHITE)
///                   color[v] = GRAY
///               else                             (u,v) is a non - tree edge
///                   if (color[v] = GRAY)         (u,v) has a gray target
///                       ...
///                   else if (color[v] = BLACK)   (u,v) has a black target
///                       ...
///           end for
///           color[u] := BLACK                    finish vertex u
///       end while
///
/// If an exception is raised inside the callback function, the graph traversal
/// will be stopped immediately. You can exploit this to exit early by raising a
/// :class:`~retworkx.visit.StopSearch` exception, in which case the search function
/// will return but without raising back the exception. You can also prune part of the
/// search tree by raising :class:`~retworkx.visit.PruneSearch`.
///
/// In the following example we keep track of the tree edges:
///
/// .. jupyter-execute::
///
///        import retworkx
///        from retworkx.visit import BFSVisitor
///   
///        class TreeEdgesRecorder(BFSVisitor):
///
///            def __init__(self):
///                self.edges = []
///
///            def tree_edge(self, edge):
///                self.edges.append(edge)
///
///        graph = retworkx.PyGraph()
///        graph.extend_from_edge_list([(1, 3), (0, 1), (2, 1), (0, 2)])
///        vis = TreeEdgesRecorder()
///        retworkx.bfs_search(graph, [0], vis)
///        print('Tree edges:', vis.edges)
///
/// .. note::
///
///     Graph can **not** be mutated while traversing.
///
/// :param PyGraph graph: The graph to be used.
/// :param List[int] source: An optional list of node indices to use as the starting nodes
///     for the breadth-first search. If this is not specified then a source
///     will be chosen arbitrarly and repeated until all components of the
///     graph are searched.
/// :param visitor: A visitor object that is invoked at the event points inside the
///     algorithm. This should be a subclass of :class:`~retworkx.visit.BFSVisitor`.
#[pyfunction]
#[pyo3(text_signature = "(graph, source, visitor)")]
pub fn graph_bfs_search(
    py: Python,
    graph: &graph::PyGraph,
    source: Option<Vec<usize>>,
    visitor: PyBfsVisitor,
) -> PyResult<()> {
    let starts: Vec<_> = match source {
        Some(nx) => nx.into_iter().map(NodeIndex::new).collect(),
        None => graph.graph.node_indices().collect(),
    };

    breadth_first_search(&graph.graph, starts, |event| {
        bfs_handler(py, &visitor, event)
    })?;

    Ok(())
}
