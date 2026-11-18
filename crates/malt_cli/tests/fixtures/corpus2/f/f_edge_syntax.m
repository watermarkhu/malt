%% Edge-case syntax tour
% A script that mixes unusual but valid MATLAB constructs. Run it section
% by section while reading the output.

clear; close all; clc
x = [4 8 15 16 23 42];
disp hello

%% End as an index and one-line if
y = x(end);   % 'end' used as an index
if y > 10, z = 1, else, z = 0, end
w = 0;
if numel(x), w = 1, end;;

%% Switch on lowercase text, and a cell of numeric cases
s = 'Blue';
switch lower(s)
    case {'red', 'green'}
        colorCode = 1;
    case 'blue'
        colorCode = 2;
    otherwise
        colorCode = 0;
end

n = 2;
switch n
    case {1, 2}
        label = 'low';
    otherwise
        label = 'high';
end

%% Loops, including an empty range and a column iterator
for k = [1 2 3]'
    fprintf('k = %d\n', k);
end
for j = 1:0
    disp('never runs')
end

%% Matrix literal across lines, then growing it by a column
A = [1  2  3
     4  5  6;
     7  8  9];
col = [10; 11; 12];
A(:, end+1) = col;

%% Names that look like keywords or options
endpoint = 'https://example.com/api';
format_str = '%6.2f\n';
ifelse = 3;
fprintf(format_str, ifelse);

%% Automatic struct creation through nested indexing
cfg.x.y(2).z = 5;
fprintf('cfg.x.y has %d elements\n', numel(cfg.x.y));

%% Plotting shorthand
hold on; grid on
plot(x, 'o-')
hold off

%% Local helper defined at the end of the script
scaled = scale_by(x, 2);
disp(scaled)

function out = scale_by(in, factor)
%SCALE_BY Multiply a vector by a factor.
out = in .* factor;
end
