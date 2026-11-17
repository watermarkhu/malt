function [moves, filled] = hanoi_flood(n, grid, seed)
%HANOI_FLOOD Towers of Hanoi move list, and a flood fill of a character grid.
%   [moves, filled] = HANOI_FLOOD(n, grid, seed) returns the move list for n disks
%   and the grid after flooding from seed = [row col].

moves = {};
moves = hanoi(n, 'A', 'C', 'B', moves);

filled = grid;
stack = seed;
visited = false(size(grid));
while ~isempty(stack)
    p = stack(end, :);
    stack(end, :) = [];
    r = p(1); c = p(2);
    if r < 1 || c < 1 || r > size(grid, 1) || c > size(grid, 2)
        continue
    end
    if visited(r, c) || grid(r, c) == '#'
        continue
    end
    visited(r, c) = true;
    filled(r, c) = '~';
    stack = [stack; r+1, c; r-1, c; r, c+1; r, c-1];
end
nVisited = nnz(visited);
fprintf('filled %d cells\n', nVisited);
end

function moves = hanoi(n, from, to, via, moves)
if n == 0
    return
end
moves = hanoi(n - 1, from, via, to, moves);
moves{end+1} = sprintf('%s->%s', from, to);
moves = hanoi(n - 1, via, to, from, moves);
end

function depth = nest_depth(c)
depth = 0;
if iscell(c)
    for k = 1:numel(c)
        d = nest_depth(c{k});
        depth = max(depth, d + 1);
    end
end
end

function count = count_odd(v)
count = 0;
k = 1;
while true
    if k > numel(v)
        break
    end
    if mod(v(k), 2) == 0
        k = k + 1;
        continue
    end
    count = count + 1;
    k = k + 1;
end
end
