%% Conway's Game of Life on a torus, with an animated display
% Parameters are at the top so they are easy to change.
% Press the Stop button in the figure to end the run early.

clear; close all;

%% Parameters
boardSize = [60 80];    % rows, columns
density = 0.25;         % initial fraction of live cells
maxGen = 500;           % generation limit
pauseTime = 0.05;       % seconds between frames
seed = 7;

%% Initial state
rng(seed);
board = rand(boardSize) < density;

% Neighbour kernel: the eight neighbours of every cell
kernel = [1 1 1; 1 0 1; 1 1 1];

%% Figure with a stop button
fig = figure('Name', 'Life', 'Color', 'w', 'NumberTitle', 'off');
set(fig, 'UserData', false);
uicontrol(fig, 'Style', 'pushbutton', 'String', 'Stop', ...
    'Units', 'normalized', 'Position', [0.02 0.02 0.12 0.06], ...
    'Callback', @(src, ~) set(ancestor(src, 'figure'), 'UserData', true));

h = imagesc(board);
axis image off;
colormap(gray);

%% Main loop
gen = 0;
population = zeros(maxGen, 1);

while gen < maxGen && ~get(fig, 'UserData')
    gen = gen + 1;

    % Count live neighbours with wrap-around (torus) boundaries
    padded = padarray(double(board), [1 1], 'circular');
    counts = conv2(padded, kernel, 'valid');

    % Birth on exactly 3 neighbours, survival on 2 or 3
    born = ~board & counts == 3;
    survive = board & (counts == 2 | counts == 3);
    board = born | survive;

    population(gen) = nnz(board);
    set(h, 'CData', board);
    title(sprintf('Generation %d, population %d', gen, population(gen)));
    drawnow;
    pause(pauseTime);

    % Stop early if the pattern has died out
    if population(gen) == 0
        fprintf('Extinct at generation %d.\n', gen);
        break
    end
end

population = population(1:gen);
fprintf('Ran %d generations; final population %d.\n', gen, population(end));

figure('Color', 'w');
plot(population, 'k-');
xlabel('Generation');
ylabel('Live cells');
grid on;
