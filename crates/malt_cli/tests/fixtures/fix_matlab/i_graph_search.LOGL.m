function result = i_graph_search(adj, start_node, goal_node)
% I_GRAPH_SEARCH  BFS, DFS and Dijkstra on an adjacency matrix
%   result = i_graph_search(adj, start_node, goal_node)

if nargin < 3
    goal_node = size(adj, 1);
end
n = size(adj, 1);
result = struct();
result.bfs = bfs_path(adj, start_node, goal_node);
result.dfs_order = dfs_order(adj, start_node);
[d, prev] = dijkstra(adj, start_node);
result.dijkstra_dist = d;
result.dijkstra_path = rebuild_path(prev, start_node, goal_node);
result.n = n;
result.map_bfs = bfs_map(build_map(adj), start_node);
fprintf('bfs path has %d nodes\n', length(result.bfs));
end

function p = bfs_path(adj, s, t)
% breadth first search; the queue is a plain array
n = size(adj, 1);
visited = false(1, n);
parent = zeros(1, n);
queue = s;
visited(s) = true;
p = [];
while ~isempty(queue)
    u = queue(1);
    queue(1) = [];
    if u == t
        break;
    end
    for v = 1:n
        if adj(u, v) ~= 0 && ~visited(v)
            visited(v) = true;
            parent(v) = u;
            queue(end+1) = v;
        end
    end
end
if ~visited(t)
    return;
end
p = t;
while p(1) ~= s
    p = [parent(p(1)), p];
end
end

function order = dfs_order(adj, s)
% depth first search with an explicit stack
n = size(adj, 1);
visited = false(n, 1);
order = [];
stack = s;
while length(stack) > 0
    u = stack(end);
    stack(end) = [];
    if visited(u)
        continue
    end
    visited(u) = 1;
    order = [order u];
    for v = n:-1:1
        if adj(u, v) && visited(v) == 0
            stack = [stack v];
        end
    end
end
end

function [dist, prev] = dijkstra(adj, s)
% dijkstra with a linear scan instead of a heap
n = size(adj, 1);
dist = inf(1, n);
dist(s) = 0;
prev = zeros(1, n);
done = false(1, n);
for iter = 1:n
    tmp = dist;
    tmp(done) = inf;
    [best, u] = min(tmp);
    if isinf(best)
        break
    end
    done(u) = true;
    for v = 1:n
        w = adj(u, v);
        if w > 0 && dist(u) + w < dist(v)
            dist(v) = dist(u) + w;
            prev(v) = u;
        end
    end
end
end

function path = rebuild_path(prev, s, t)
% walk the predecessor array backwards from t
path = t;
cur = t;
while cur ~= s
    if prev(cur) == 0
        path = [];
        return
    end
    cur = prev(cur);
    path = [cur path];
end
end

function m = build_map(adj)
% convert the matrix into an adjacency list held in a containers.Map
m = containers.Map('KeyType', 'double', 'ValueType', 'any');
n = size(adj, 1);
for i = 1:n
    nbrs = find(adj(i, :) > 0);
    m(i) = nbrs;
end
end

function order = bfs_map(m, s)
% breadth first search over the adjacency list
queue = {s};
seen = containers.Map('KeyType', 'double', 'ValueType', 'logical');
seen(s) = true;
order = [];
while ~isempty(queue)
    u = queue{1};
    queue(1) = [];
    order(end+1) = u;
    if isKey(m, u)
        nb = m(u);
    else
        nb = [];
    end
    for k = 1:numel(nb)
        v = nb(k);
        if ~isKey(seen, v)
            seen(v) = true;
            queue{end+1} = v;
        end
    end
end
end
