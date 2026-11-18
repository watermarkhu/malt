function i_game_of_life_and_maze(seed)
% I_GAME_OF_LIFE_AND_MAZE  life on a torus and a recursive maze solver
%   i_game_of_life_and_maze(seed)

if nargin < 1
    seed = 42;
end
rng(seed);

% --- conway's life on a torus ---
grid = rand(8, 8) > 0.7;
for gen = 1:5
    grid = life_step(grid);
    render(grid, gen);
end

% --- maze solving ---
maze = [0 0 1 0 0;
        1 0 1 0 1;
        0 0 0 0 0;
        0 1 1 1 0;
        0 0 0 1 0];
print_maze(maze);
visited = false(size(maze));
found = solve_maze(maze, 1, 1, 5, 5, visited);
fprintf('maze solvable: %d\n', found);
end

function next = life_step(g)
% one generation, neighbours counted with circshift so the edges wrap
[r, c] = size(g);
n = zeros(r, c);
for dr = -1:1
    for dc = -1:1
        if dr == 0 && dc == 0
            continue
        end
        n = n + circshift(g, [dr dc]);
    end
end
next = (g & (n == 2 | n == 3)) | (~g & n == 3);
end

function render(g, gen)
% print the grid with # for alive and . for dead
fprintf('generation %d\n', gen);
for i = 1:size(g, 1)
    row = '';
    for j = 1:size(g, 2)
        if g(i, j)
            row = [row '#'];
        else
            row = [row '.'];
        end
    end
    fprintf('%s\n', row);
end
end

function print_maze(m)
% walls are 1, open cells are 0
for i = 1:size(m, 1)
    line = '';
    for j = 1:size(m, 2)
        if m(i, j) == 1
            line = [line '█'];
        else
            line = [line ' '];
        end
    end
    disp(line);
end
end

function ok = solve_maze(m, r, c, tr, tc, visited)
% depth first search, returns true when the target is reachable
ok = false;
if r < 1 || c < 1 || r > size(m, 1) || c > size(m, 2)
    return;
end
if m(r, c) == 1 || visited(r, c)
    return;
end
if r == tr && c == tc
    ok = true;
    return;
end
visited(r, c) = true;
ok = solve_maze(m, r+1, c, tr, tc, visited) || ...
     solve_maze(m, r-1, c, tr, tc, visited) || ...
     solve_maze(m, r, c+1, tr, tc, visited) || ...
     solve_maze(m, r, c-1, tr, tc, visited);
end

function n = count_neighbors(g, i, j)
% live neighbours of one cell on a torus
[r, c] = size(g);
n = 0;
for di = -1:1
    for dj = -1:1
        if di == 0 && dj == 0
            continue
        end
        ii = mod(i + di - 1, r) + 1;
        jj = mod(j + dj - 1, c) + 1;
        n = n + g(ii, jj);
    end
end
end
